//! Provider-free, read-only classification of frozen C1 artifact prefixes.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum C1OfflineState {
    Pristine,
    IntentOnlyFrozen,
    MutationDispatchOutcomeUnknown,
    IncompleteFrozenAfterAction,
    TerminalPairTimingUnproven,
    TerminalPairClaimsLate,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum C1ArtifactReadError {
    UnsupportedPlatform,
    Unreadable,
    UnsafeDirectory,
    UnsafeEntry,
    IdentityChanged,
    InvalidResidue,
}

#[allow(dead_code)]
pub(crate) fn classify_c1_artifact_directory(
    path: &Path,
) -> Result<C1OfflineState, C1ArtifactReadError> {
    #[cfg(target_os = "macos")]
    {
        macos::classify(path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err(C1ArtifactReadError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{C1ArtifactReadError, C1OfflineState};
    use crate::native_document::{
        require_no_extended_acl, strict_json_value_from_slice_categorized, NativeSuccessorFamily,
    };
    use crate::native_successor_policy::{
        C1ExecutionId, C1Phase, C1PhaseArtifactPolicy, C1PolicyId, C1Sha256,
    };
    use serde::Deserialize;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::collections::{BTreeMap, BTreeSet};
    use std::ffi::{CStr, CString};
    use std::fs::{self, File, Metadata, OpenOptions};
    use std::io::{Read, Seek, SeekFrom};
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, FromRawFd, RawFd};
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    use std::path::Path;

    const MAX_PRIMARY_BYTES: u64 = 1_048_576;
    const SIDECAR_BYTES: u64 = 65;
    const MAX_ENTRIES: usize = 8;
    const MAX_JSON_INTEGER: u64 = 9_007_199_254_740_991;
    const ENTRY_OPEN_FLAGS: libc::c_int =
        libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC;

    const INTENT: &str = "execution-intent.json";
    const INTENT_SIDECAR: &str = "execution-intent.json.sha256";
    const DISPATCH: &str = "mutation-dispatch-001.json";
    const DISPATCH_SIDECAR: &str = "mutation-dispatch-001.json.sha256";
    const ACTION: &str = "phase-action-receipt.json";
    const ACTION_SIDECAR: &str = "phase-action-receipt.json.sha256";
    const TERMINAL: &str = "terminal-receipt.json";
    const TERMINAL_SIDECAR: &str = "terminal-receipt.json.sha256";

    const ALLOWED_NAMES: [&str; 8] = [
        INTENT,
        INTENT_SIDECAR,
        DISPATCH,
        DISPATCH_SIDECAR,
        ACTION,
        ACTION_SIDECAR,
        TERMINAL,
        TERMINAL_SIDECAR,
    ];

    pub(super) fn classify(path: &Path) -> Result<C1OfflineState, C1ArtifactReadError> {
        let directory = RetainedDirectory::open(path)?;
        let enumeration = directory.enumerate();
        let directory_validation = directory.revalidate();
        let names = with_directory_precedence(enumeration, directory_validation)?;
        if names.len() > MAX_ENTRIES {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        if names
            .iter()
            .any(|name| !ALLOWED_NAMES.contains(&name.as_str()))
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }

        let mut entries = BTreeMap::new();
        let mut retained_entries = BTreeMap::new();
        for name in &names {
            let limit = if name.ends_with(".sha256") {
                SIDECAR_BYTES
            } else {
                MAX_PRIMARY_BYTES
            };
            let entry = directory.read_entry(name, limit);
            let directory_validation = directory.revalidate();
            let (bytes, retained) = with_directory_precedence(entry, directory_validation)?;
            entries.insert(name.clone(), bytes);
            retained_entries.insert(name.clone(), retained);
        }
        let classification = classify_entries(&entries);

        #[cfg(test)]
        test_hook::checkpoint("after-first-sweep");

        let enumeration = directory.require_same_names(&names);
        let directory_validation = directory.revalidate();
        with_directory_precedence(enumeration, directory_validation)?;
        for name in &names {
            let entry_validation = retained_entries
                .get_mut(name)
                .expect("retained entry exists for every enumerated name")
                .revalidate(
                    directory.file.as_raw_fd(),
                    directory.effective_uid,
                    &entries[name],
                );
            let directory_validation = directory.revalidate();
            with_directory_precedence(entry_validation, directory_validation)?;
        }
        let enumeration = directory.require_same_names(&names);
        let directory_validation = directory.revalidate();
        with_directory_precedence(enumeration, directory_validation)?;
        classification
    }

    fn with_directory_precedence<T>(
        operation: Result<T, C1ArtifactReadError>,
        directory_validation: Result<(), C1ArtifactReadError>,
    ) -> Result<T, C1ArtifactReadError> {
        if directory_validation == Err(C1ArtifactReadError::UnsafeDirectory) {
            return Err(C1ArtifactReadError::UnsafeDirectory);
        }
        let value = operation?;
        directory_validation?;
        Ok(value)
    }

    struct RetainedDirectory<'a> {
        source_path: &'a Path,
        canonical_path: std::path::PathBuf,
        file: File,
        identity: FileIdentity,
        effective_uid: u32,
    }

    impl<'a> RetainedDirectory<'a> {
        fn open(path: &'a Path) -> Result<Self, C1ArtifactReadError> {
            let path_metadata =
                fs::symlink_metadata(path).map_err(|_| C1ArtifactReadError::Unreadable)?;
            let effective_uid = unsafe { libc::geteuid() };
            if !directory_is_safe(&path_metadata, effective_uid) {
                return Err(C1ArtifactReadError::UnsafeDirectory);
            }
            let canonical_path = path
                .canonicalize()
                .map_err(|_| C1ArtifactReadError::Unreadable)?;
            if canonical_path != path {
                return Err(C1ArtifactReadError::UnsafeDirectory);
            }

            let mut options = OpenOptions::new();
            options
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
            let file = options
                .open(path)
                .map_err(|_| C1ArtifactReadError::Unreadable)?;
            let opened_metadata = file
                .metadata()
                .map_err(|_| C1ArtifactReadError::Unreadable)?;
            if !directory_is_safe(&opened_metadata, effective_uid)
                || require_no_extended_acl(&file, "C1 artifact directory").is_err()
            {
                return Err(C1ArtifactReadError::UnsafeDirectory);
            }
            if FileIdentity::from_metadata(&path_metadata)
                != FileIdentity::from_metadata(&opened_metadata)
            {
                return Err(C1ArtifactReadError::IdentityChanged);
            }
            let directory = Self {
                source_path: path,
                canonical_path,
                file,
                identity: FileIdentity::from_metadata(&opened_metadata),
                effective_uid,
            };
            directory.revalidate()?;
            Ok(directory)
        }

        fn enumerate(&self) -> Result<BTreeSet<String>, C1ArtifactReadError> {
            let dot = c".";
            let fd = unsafe {
                libc::openat(
                    self.file.as_raw_fd(),
                    dot.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                return Err(C1ArtifactReadError::Unreadable);
            }
            let stream = unsafe { libc::fdopendir(fd) };
            if stream.is_null() {
                unsafe {
                    libc::close(fd);
                }
                return Err(C1ArtifactReadError::Unreadable);
            }

            let result = (|| {
                let mut names = BTreeSet::new();
                loop {
                    unsafe {
                        *libc::__error() = 0;
                    }
                    let entry = unsafe { libc::readdir(stream) };
                    if entry.is_null() {
                        let error = unsafe { *libc::__error() };
                        if error != 0 {
                            return Err(C1ArtifactReadError::Unreadable);
                        }
                        break;
                    }
                    let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
                    if name == b"." || name == b".." {
                        continue;
                    }
                    let name = std::str::from_utf8(name)
                        .map_err(|_| C1ArtifactReadError::InvalidResidue)?;
                    if names.len() == MAX_ENTRIES {
                        return Err(C1ArtifactReadError::InvalidResidue);
                    }
                    if !names.insert(name.to_string()) {
                        return Err(C1ArtifactReadError::InvalidResidue);
                    }
                }
                Ok(names)
            })();
            let close_result = unsafe { libc::closedir(stream) };
            if close_result != 0 {
                return Err(C1ArtifactReadError::Unreadable);
            }
            result
        }

        fn read_entry(
            &self,
            name: &str,
            maximum_bytes: u64,
        ) -> Result<(Vec<u8>, RetainedEntry), C1ArtifactReadError> {
            let name_c = CString::new(name).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
            let before = stat_at(self.file.as_raw_fd(), &name_c)
                .map_err(|_| C1ArtifactReadError::Unreadable)?;
            if !entry_is_safe_stat(&before, self.effective_uid) {
                return Err(C1ArtifactReadError::UnsafeEntry);
            }
            let length =
                u64::try_from(before.st_size).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
            if length == 0 || length > maximum_bytes {
                return Err(C1ArtifactReadError::InvalidResidue);
            }

            #[cfg(test)]
            test_hook::checkpoint(&format!("before-open:{name}"));

            let fd =
                unsafe { libc::openat(self.file.as_raw_fd(), name_c.as_ptr(), ENTRY_OPEN_FLAGS) };
            if fd < 0 {
                return match stat_at(self.file.as_raw_fd(), &name_c) {
                    Ok(after) if !entry_is_safe_stat(&after, self.effective_uid) => {
                        Err(C1ArtifactReadError::UnsafeEntry)
                    }
                    Ok(after)
                        if FileIdentity::from_stat(&before) == FileIdentity::from_stat(&after) =>
                    {
                        Err(C1ArtifactReadError::Unreadable)
                    }
                    Ok(_) | Err(()) => Err(C1ArtifactReadError::IdentityChanged),
                };
            }
            let mut file = unsafe { File::from_raw_fd(fd) };
            let expected_identity = FileIdentity::from_stat(&before);
            revalidate_entry_boundary(
                &file,
                self.file.as_raw_fd(),
                &name_c,
                self.effective_uid,
                expected_identity,
            )?;

            #[cfg(test)]
            test_hook::checkpoint(&format!("before-first-sweep-read:{name}"));

            let first = read_exact_snapshot(&mut file, length, expected_identity);
            let first_boundary = revalidate_entry_boundary(
                &file,
                self.file.as_raw_fd(),
                &name_c,
                self.effective_uid,
                expected_identity,
            );
            first_boundary?;
            let first = first?;
            let first_sha = Sha256::digest(&first);

            let seek_result = file
                .seek(SeekFrom::Start(0))
                .map_err(|_| C1ArtifactReadError::Unreadable);
            let seek_boundary = revalidate_entry_boundary(
                &file,
                self.file.as_raw_fd(),
                &name_c,
                self.effective_uid,
                expected_identity,
            );
            seek_boundary?;
            seek_result?;

            #[cfg(test)]
            test_hook::checkpoint(&format!("before-second-first-sweep-read:{name}"));

            let second = read_exact_snapshot(&mut file, length, expected_identity);

            #[cfg(test)]
            test_hook::checkpoint(name);

            let second_boundary = revalidate_entry_boundary(
                &file,
                self.file.as_raw_fd(),
                &name_c,
                self.effective_uid,
                expected_identity,
            );
            second_boundary?;
            let second = second?;
            let second_sha = Sha256::digest(&second);
            if first != second || first_sha != second_sha {
                return Err(C1ArtifactReadError::IdentityChanged);
            }
            let digest = first_sha.into();
            Ok((
                first,
                RetainedEntry {
                    name: name_c,
                    file,
                    identity: expected_identity,
                    length,
                    digest,
                },
            ))
        }

        fn require_same_names(
            &self,
            expected: &BTreeSet<String>,
        ) -> Result<(), C1ArtifactReadError> {
            match self.enumerate() {
                Ok(observed) if observed == *expected => Ok(()),
                Ok(_) | Err(C1ArtifactReadError::InvalidResidue) => {
                    Err(C1ArtifactReadError::IdentityChanged)
                }
                Err(error) => Err(error),
            }
        }

        fn revalidate(&self) -> Result<(), C1ArtifactReadError> {
            let handle_metadata = self
                .file
                .metadata()
                .map_err(|_| C1ArtifactReadError::Unreadable)?;
            if !directory_is_safe(&handle_metadata, self.effective_uid) {
                return Err(C1ArtifactReadError::UnsafeDirectory);
            }
            if require_no_extended_acl(&self.file, "C1 artifact directory").is_err() {
                return Err(C1ArtifactReadError::UnsafeDirectory);
            }
            let path_metadata = fs::symlink_metadata(self.source_path)
                .map_err(|_| C1ArtifactReadError::IdentityChanged)?;
            if !directory_is_safe(&path_metadata, self.effective_uid) {
                return Err(C1ArtifactReadError::UnsafeDirectory);
            }
            let canonical_now = self
                .source_path
                .canonicalize()
                .map_err(|_| C1ArtifactReadError::IdentityChanged)?;
            if self.identity != FileIdentity::from_metadata(&handle_metadata)
                || self.identity != FileIdentity::from_metadata(&path_metadata)
                || canonical_now != self.canonical_path
            {
                return Err(C1ArtifactReadError::IdentityChanged);
            }
            Ok(())
        }
    }

    struct RetainedEntry {
        name: CString,
        file: File,
        identity: FileIdentity,
        length: u64,
        digest: [u8; 32],
    }

    impl RetainedEntry {
        fn revalidate(
            &mut self,
            directory_fd: RawFd,
            effective_uid: u32,
            expected_bytes: &[u8],
        ) -> Result<(), C1ArtifactReadError> {
            revalidate_entry_boundary(
                &self.file,
                directory_fd,
                &self.name,
                effective_uid,
                self.identity,
            )?;
            let seek_result = self
                .file
                .seek(SeekFrom::Start(0))
                .map_err(|_| C1ArtifactReadError::Unreadable);
            let seek_boundary = revalidate_entry_boundary(
                &self.file,
                directory_fd,
                &self.name,
                effective_uid,
                self.identity,
            );
            seek_boundary?;
            seek_result?;

            #[cfg(test)]
            test_hook::checkpoint(&format!(
                "before-second-sweep-read:{}",
                self.name
                    .to_str()
                    .expect("retained entry name is valid UTF-8")
            ));

            let observed = read_exact_snapshot(&mut self.file, self.length, self.identity);
            let post_read_validation = revalidate_entry_boundary(
                &self.file,
                directory_fd,
                &self.name,
                effective_uid,
                self.identity,
            );
            post_read_validation?;
            let observed = observed?;
            let observed_digest: [u8; 32] = Sha256::digest(&observed).into();
            if observed != expected_bytes || observed_digest != self.digest {
                return Err(C1ArtifactReadError::IdentityChanged);
            }
            Ok(())
        }
    }

    fn revalidate_entry_boundary(
        file: &File,
        directory_fd: RawFd,
        name: &CStr,
        effective_uid: u32,
        identity: FileIdentity,
    ) -> Result<(), C1ArtifactReadError> {
        let handle = file
            .metadata()
            .map_err(|_| C1ArtifactReadError::Unreadable)?;
        if !entry_is_safe_metadata(&handle, effective_uid)
            || require_no_extended_acl(file, "C1 artifact entry").is_err()
        {
            return Err(C1ArtifactReadError::UnsafeEntry);
        }
        let path = stat_at(directory_fd, name).map_err(|_| C1ArtifactReadError::IdentityChanged)?;
        if !entry_is_safe_stat(&path, effective_uid) {
            return Err(C1ArtifactReadError::UnsafeEntry);
        }
        if identity != FileIdentity::from_metadata(&handle)
            || identity != FileIdentity::from_stat(&path)
        {
            return Err(C1ArtifactReadError::IdentityChanged);
        }
        Ok(())
    }

    fn read_exact_snapshot(
        file: &mut File,
        length: u64,
        expected_identity: FileIdentity,
    ) -> Result<Vec<u8>, C1ArtifactReadError> {
        let length = usize::try_from(length).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        let mut bytes = vec![0; length];
        if file.read_exact(&mut bytes).is_err() {
            return match file.metadata() {
                Ok(metadata) if FileIdentity::from_metadata(&metadata) != expected_identity => {
                    Err(C1ArtifactReadError::IdentityChanged)
                }
                _ => Err(C1ArtifactReadError::Unreadable),
            };
        }
        let mut trailing = [0_u8; 1];
        if file
            .read(&mut trailing)
            .map_err(|_| C1ArtifactReadError::Unreadable)?
            != 0
        {
            return Err(C1ArtifactReadError::IdentityChanged);
        }
        Ok(bytes)
    }

    fn stat_at(directory_fd: RawFd, name: &CStr) -> Result<libc::stat, ()> {
        let mut value = MaybeUninit::<libc::stat>::uninit();
        let result = unsafe {
            libc::fstatat(
                directory_fd,
                name.as_ptr(),
                value.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result != 0 {
            return Err(());
        }
        Ok(unsafe { value.assume_init() })
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct FileIdentity {
        device: u64,
        inode: u64,
        length: u64,
        modified_seconds: i64,
        modified_nanoseconds: i64,
        changed_seconds: i64,
        changed_nanoseconds: i64,
    }

    impl FileIdentity {
        fn from_metadata(metadata: &Metadata) -> Self {
            Self {
                device: metadata.dev(),
                inode: metadata.ino(),
                length: metadata.size(),
                modified_seconds: metadata.mtime(),
                modified_nanoseconds: metadata.mtime_nsec(),
                changed_seconds: metadata.ctime(),
                changed_nanoseconds: metadata.ctime_nsec(),
            }
        }

        fn from_stat(metadata: &libc::stat) -> Self {
            Self {
                device: metadata.st_dev as u64,
                inode: metadata.st_ino,
                length: u64::try_from(metadata.st_size).unwrap_or(u64::MAX),
                modified_seconds: metadata.st_mtime,
                modified_nanoseconds: metadata.st_mtime_nsec,
                changed_seconds: metadata.st_ctime,
                changed_nanoseconds: metadata.st_ctime_nsec,
            }
        }
    }

    fn directory_is_safe(metadata: &Metadata, effective_uid: u32) -> bool {
        metadata.is_dir()
            && metadata.uid() == effective_uid
            && metadata.permissions().mode() & 0o7777 == 0o700
    }

    fn entry_is_safe_metadata(metadata: &Metadata, effective_uid: u32) -> bool {
        metadata.is_file()
            && metadata.uid() == effective_uid
            && metadata.nlink() == 1
            && metadata.permissions().mode() & 0o7777 == 0o600
    }

    fn entry_is_safe_stat(metadata: &libc::stat, effective_uid: u32) -> bool {
        metadata.st_mode & libc::S_IFMT == libc::S_IFREG
            && metadata.st_uid == effective_uid
            && metadata.st_nlink == 1
            && metadata.st_mode & 0o7777 == 0o600
    }

    fn classify_entries(
        entries: &BTreeMap<String, Vec<u8>>,
    ) -> Result<C1OfflineState, C1ArtifactReadError> {
        if entries.is_empty() {
            return Ok(C1OfflineState::Pristine);
        }
        if entries
            .keys()
            .any(|name| !ALLOWED_NAMES.contains(&name.as_str()))
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }

        let intent_present = pair_present(entries, INTENT, INTENT_SIDECAR)?;
        let dispatch_present = pair_present(entries, DISPATCH, DISPATCH_SIDECAR)?;
        let action_present = pair_present(entries, ACTION, ACTION_SIDECAR)?;
        let terminal_present = pair_present(entries, TERMINAL, TERMINAL_SIDECAR)?;
        if !intent_present || (!action_present && terminal_present) {
            return Err(C1ArtifactReadError::InvalidResidue);
        }

        let intent_sha = verify_pair(entries, INTENT, INTENT_SIDECAR)?;
        let intent = parse_intent(entries.get(INTENT).expect("complete intent pair"))?;
        if !action_present {
            if terminal_present || (dispatch_present && !intent.requires_dispatch) {
                return Err(C1ArtifactReadError::InvalidResidue);
            }
            if dispatch_present {
                verify_pair(entries, DISPATCH, DISPATCH_SIDECAR)?;
                let dispatch =
                    parse_dispatch(entries.get(DISPATCH).expect("complete dispatch pair"))?;
                validate_dispatch(&intent, &dispatch, &intent_sha)?;
                return Ok(C1OfflineState::MutationDispatchOutcomeUnknown);
            }
            return Ok(C1OfflineState::IntentOnlyFrozen);
        }

        if dispatch_present != intent.requires_dispatch {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        let dispatch = if dispatch_present {
            let digest = verify_pair(entries, DISPATCH, DISPATCH_SIDECAR)?;
            let artifact = parse_dispatch(entries.get(DISPATCH).expect("complete dispatch pair"))?;
            validate_dispatch(&intent, &artifact, &intent_sha)?;
            Some((artifact, digest))
        } else {
            None
        };
        let action_sha = verify_pair(entries, ACTION, ACTION_SIDECAR)?;
        let action = parse_action(entries.get(ACTION).expect("complete action pair"))?;
        validate_action(&intent, dispatch.as_ref(), &action, &intent_sha)?;
        if !terminal_present {
            return Ok(C1OfflineState::IncompleteFrozenAfterAction);
        }

        let _terminal_sha = verify_pair(entries, TERMINAL, TERMINAL_SIDECAR)?;
        let terminal = parse_terminal(entries.get(TERMINAL).expect("complete terminal pair"))?;
        validate_terminal(
            &intent,
            dispatch.as_ref(),
            &action,
            &terminal,
            &intent_sha,
            &action_sha,
        )?;
        match terminal.timing_claim {
            TimingClaim::Timely => Ok(C1OfflineState::TerminalPairTimingUnproven),
            TimingClaim::Late => Ok(C1OfflineState::TerminalPairClaimsLate),
        }
    }

    fn pair_present(
        entries: &BTreeMap<String, Vec<u8>>,
        primary: &str,
        sidecar: &str,
    ) -> Result<bool, C1ArtifactReadError> {
        match (entries.contains_key(primary), entries.contains_key(sidecar)) {
            (true, true) => Ok(true),
            (false, false) => Ok(false),
            _ => Err(C1ArtifactReadError::InvalidResidue),
        }
    }

    fn verify_pair(
        entries: &BTreeMap<String, Vec<u8>>,
        primary: &str,
        sidecar: &str,
    ) -> Result<String, C1ArtifactReadError> {
        let primary = entries
            .get(primary)
            .ok_or(C1ArtifactReadError::InvalidResidue)?;
        let sidecar = entries
            .get(sidecar)
            .ok_or(C1ArtifactReadError::InvalidResidue)?;
        if sidecar.len() != SIDECAR_BYTES as usize || sidecar.last() != Some(&b'\n') {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        let spelling =
            std::str::from_utf8(&sidecar[..64]).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        if C1Sha256::parse(spelling).is_none() {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        let digest = C1Sha256::from_digest_bytes(Sha256::digest(primary).into());
        if digest.as_str() != spelling {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(spelling.to_string())
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Envelope {
        family: EnvelopeFamily,
        schema_version: u32,
        payload: Value,
    }

    #[derive(Deserialize)]
    enum EnvelopeFamily {
        #[serde(rename = "native_stale_isolated_v1")]
        StaleIsolated,
        #[serde(rename = "native_correction_v1")]
        Correction,
    }

    impl EnvelopeFamily {
        fn native(self) -> NativeSuccessorFamily {
            match self {
                Self::StaleIsolated => NativeSuccessorFamily::StaleIsolatedV1,
                Self::Correction => NativeSuccessorFamily::CorrectionV1,
            }
        }
    }

    #[derive(Deserialize)]
    enum IntentKind {
        #[serde(rename = "execution_intent")]
        Exact,
    }

    #[derive(Deserialize)]
    enum AuditKind {
        #[serde(rename = "audit")]
        Exact,
    }

    #[derive(Deserialize)]
    enum TerminalKind {
        #[serde(rename = "terminal_receipt")]
        Exact,
    }

    #[derive(Deserialize)]
    enum DispatchKind {
        #[serde(rename = "mutation_dispatch")]
        Exact,
    }

    #[derive(Deserialize)]
    enum ActionKind {
        #[serde(rename = "phase_action_receipt")]
        Exact,
    }

    #[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum Outcome {
        Success,
        Failure,
    }

    #[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum TimingClaim {
        Timely,
        Late,
    }

    #[derive(Deserialize)]
    struct RequiredNullableString(Option<String>);

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct IntentPayload {
        document_kind: IntentKind,
        artifact_schema_version: u64,
        execution_id: String,
        plan_sha256: String,
        phase: String,
        sequence: u64,
        previous_artifact_sha256: RequiredNullableString,
        phase_policy_id: String,
        phase_policy_sha256: String,
        static_launch_template_sha256: String,
        declared_started_unix_ms: u64,
        declared_terminal_unix_ms: u64,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct DispatchPayload {
        document_kind: AuditKind,
        artifact_kind: DispatchKind,
        artifact_schema_version: u64,
        execution_id: String,
        plan_sha256: String,
        phase: String,
        sequence: u64,
        previous_artifact_sha256: RequiredNullableString,
        ordinal: u64,
        request_sha256: String,
        created_unix_ms: u64,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ActionPayload {
        document_kind: AuditKind,
        artifact_kind: ActionKind,
        artifact_schema_version: u64,
        execution_id: String,
        plan_sha256: String,
        phase: String,
        sequence: u64,
        previous_artifact_sha256: RequiredNullableString,
        ordinal: u64,
        request_sha256: String,
        response_projection_sha256: String,
        outcome: Outcome,
        started_unix_ms: u64,
        completed_unix_ms: u64,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct TerminalPayload {
        document_kind: TerminalKind,
        artifact_schema_version: u64,
        execution_id: String,
        plan_sha256: String,
        phase: String,
        sequence: u64,
        previous_artifact_sha256: RequiredNullableString,
        intent_sha256: String,
        dispatch_sha256: RequiredNullableString,
        action_sha256: String,
        outcome: Outcome,
        timing_claim: TimingClaim,
        completed_unix_ms: u64,
    }

    struct CommonBinding {
        family: NativeSuccessorFamily,
        execution_id: String,
        plan_sha256: String,
        phase: C1Phase,
        sequence: u64,
        previous_sha256: Option<String>,
    }

    struct IntentArtifact {
        common: CommonBinding,
        requires_dispatch: bool,
        declared_start: u64,
        declared_terminal: u64,
    }

    struct DispatchArtifact {
        common: CommonBinding,
        request_sha256: String,
        created: u64,
    }

    struct ActionArtifact {
        common: CommonBinding,
        request_sha256: String,
        outcome: Outcome,
        started: u64,
        completed: u64,
    }

    struct TerminalArtifact {
        common: CommonBinding,
        intent_sha256: String,
        dispatch_sha256: Option<String>,
        action_sha256: String,
        outcome: Outcome,
        timing_claim: TimingClaim,
        completed: u64,
    }

    fn parse_envelope(bytes: &[u8]) -> Result<(NativeSuccessorFamily, Value), C1ArtifactReadError> {
        if bytes.len() < 2 || bytes.last() != Some(&b'\n') || bytes[bytes.len() - 2] != b'}' {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        let value = strict_json_value_from_slice_categorized(bytes)
            .map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        let envelope: Envelope =
            serde_json::from_value(value).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        if envelope.schema_version != 1 || !envelope.payload.is_object() {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok((envelope.family.native(), envelope.payload))
    }

    fn parse_intent(bytes: &[u8]) -> Result<IntentArtifact, C1ArtifactReadError> {
        let (family, value) = parse_envelope(bytes)?;
        let payload: IntentPayload =
            serde_json::from_value(value).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        let _ = payload.document_kind;
        let common = common_binding(
            family,
            payload.artifact_schema_version,
            payload.execution_id,
            payload.plan_sha256,
            payload.phase,
            payload.sequence,
            payload.previous_artifact_sha256,
        )?;
        if common.sequence != 1 || common.previous_sha256.is_some() {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        let policy = C1PhaseArtifactPolicy::derive(common.family, common.phase)
            .ok_or(C1ArtifactReadError::InvalidResidue)?;
        if C1PolicyId::parse(&payload.phase_policy_id).is_none()
            || C1Sha256::parse(&payload.phase_policy_sha256).is_none()
            || C1Sha256::parse(&payload.static_launch_template_sha256).is_none()
            || payload.phase_policy_id != policy.policy_id().as_str()
            || payload.phase_policy_sha256 != policy.policy_sha256().as_str()
            || !valid_timestamp(payload.declared_started_unix_ms)
            || !valid_timestamp(payload.declared_terminal_unix_ms)
            || payload.declared_started_unix_ms >= payload.declared_terminal_unix_ms
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(IntentArtifact {
            common,
            requires_dispatch: policy.requires_dispatch(),
            declared_start: payload.declared_started_unix_ms,
            declared_terminal: payload.declared_terminal_unix_ms,
        })
    }

    fn parse_dispatch(bytes: &[u8]) -> Result<DispatchArtifact, C1ArtifactReadError> {
        let (family, value) = parse_envelope(bytes)?;
        let payload: DispatchPayload =
            serde_json::from_value(value).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        let _ = (payload.document_kind, payload.artifact_kind);
        let common = common_binding(
            family,
            payload.artifact_schema_version,
            payload.execution_id,
            payload.plan_sha256,
            payload.phase,
            payload.sequence,
            payload.previous_artifact_sha256,
        )?;
        if payload.ordinal != 1
            || C1Sha256::parse(&payload.request_sha256).is_none()
            || !valid_timestamp(payload.created_unix_ms)
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(DispatchArtifact {
            common,
            request_sha256: payload.request_sha256,
            created: payload.created_unix_ms,
        })
    }

    fn parse_action(bytes: &[u8]) -> Result<ActionArtifact, C1ArtifactReadError> {
        let (family, value) = parse_envelope(bytes)?;
        let payload: ActionPayload =
            serde_json::from_value(value).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        let _ = (payload.document_kind, payload.artifact_kind);
        let common = common_binding(
            family,
            payload.artifact_schema_version,
            payload.execution_id,
            payload.plan_sha256,
            payload.phase,
            payload.sequence,
            payload.previous_artifact_sha256,
        )?;
        if payload.ordinal != 1
            || C1Sha256::parse(&payload.request_sha256).is_none()
            || C1Sha256::parse(&payload.response_projection_sha256).is_none()
            || !valid_timestamp(payload.started_unix_ms)
            || !valid_timestamp(payload.completed_unix_ms)
            || payload.completed_unix_ms < payload.started_unix_ms
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(ActionArtifact {
            common,
            request_sha256: payload.request_sha256,
            outcome: payload.outcome,
            started: payload.started_unix_ms,
            completed: payload.completed_unix_ms,
        })
    }

    fn parse_terminal(bytes: &[u8]) -> Result<TerminalArtifact, C1ArtifactReadError> {
        let (family, value) = parse_envelope(bytes)?;
        let payload: TerminalPayload =
            serde_json::from_value(value).map_err(|_| C1ArtifactReadError::InvalidResidue)?;
        let _ = payload.document_kind;
        let common = common_binding(
            family,
            payload.artifact_schema_version,
            payload.execution_id,
            payload.plan_sha256,
            payload.phase,
            payload.sequence,
            payload.previous_artifact_sha256,
        )?;
        if C1Sha256::parse(&payload.intent_sha256).is_none()
            || payload
                .dispatch_sha256
                .0
                .as_deref()
                .is_some_and(|value| C1Sha256::parse(value).is_none())
            || C1Sha256::parse(&payload.action_sha256).is_none()
            || !valid_timestamp(payload.completed_unix_ms)
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(TerminalArtifact {
            common,
            intent_sha256: payload.intent_sha256,
            dispatch_sha256: payload.dispatch_sha256.0,
            action_sha256: payload.action_sha256,
            outcome: payload.outcome,
            timing_claim: payload.timing_claim,
            completed: payload.completed_unix_ms,
        })
    }

    fn common_binding(
        family: NativeSuccessorFamily,
        artifact_schema_version: u64,
        execution_id: String,
        plan_sha256: String,
        phase: String,
        sequence: u64,
        previous_sha256: RequiredNullableString,
    ) -> Result<CommonBinding, C1ArtifactReadError> {
        let phase = C1Phase::parse_exact(&phase).ok_or(C1ArtifactReadError::InvalidResidue)?;
        if artifact_schema_version != 1
            || C1ExecutionId::parse(&execution_id).is_none()
            || C1Sha256::parse(&plan_sha256).is_none()
            || previous_sha256
                .0
                .as_deref()
                .is_some_and(|value| C1Sha256::parse(value).is_none())
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(CommonBinding {
            family,
            execution_id,
            plan_sha256,
            phase,
            sequence,
            previous_sha256: previous_sha256.0,
        })
    }

    fn validate_dispatch(
        intent: &IntentArtifact,
        dispatch: &DispatchArtifact,
        intent_sha: &str,
    ) -> Result<(), C1ArtifactReadError> {
        if !intent.requires_dispatch
            || !common_matches(&intent.common, &dispatch.common)
            || dispatch.common.sequence != 2
            || dispatch.common.previous_sha256.as_deref() != Some(intent_sha)
            || dispatch.created < intent.declared_start
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(())
    }

    fn validate_action(
        intent: &IntentArtifact,
        dispatch: Option<&(DispatchArtifact, String)>,
        action: &ActionArtifact,
        intent_sha: &str,
    ) -> Result<(), C1ArtifactReadError> {
        let expected_previous = dispatch
            .map(|(_, digest)| digest.as_str())
            .unwrap_or(intent_sha);
        let expected_sequence = if dispatch.is_some() { 3 } else { 2 };
        if !common_matches(&intent.common, &action.common)
            || action.common.sequence != expected_sequence
            || action.common.previous_sha256.as_deref() != Some(expected_previous)
            || action.started < intent.declared_start
            || dispatch.is_some_and(|(artifact, _)| {
                artifact.created > action.started
                    || artifact.request_sha256 != action.request_sha256
            })
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(())
    }

    fn validate_terminal(
        intent: &IntentArtifact,
        dispatch: Option<&(DispatchArtifact, String)>,
        action: &ActionArtifact,
        terminal: &TerminalArtifact,
        intent_sha: &str,
        action_sha: &str,
    ) -> Result<(), C1ArtifactReadError> {
        let expected_sequence = if dispatch.is_some() { 4 } else { 3 };
        let dispatch_digest = dispatch.map(|(_, digest)| digest.as_str());
        let timing_matches = match terminal.timing_claim {
            TimingClaim::Late => terminal.completed > intent.declared_terminal,
            TimingClaim::Timely => terminal.completed <= intent.declared_terminal,
        };
        if !common_matches(&intent.common, &terminal.common)
            || terminal.common.sequence != expected_sequence
            || terminal.common.previous_sha256.as_deref() != Some(action_sha)
            || terminal.intent_sha256 != intent_sha
            || terminal.dispatch_sha256.as_deref() != dispatch_digest
            || terminal.action_sha256 != action_sha
            || terminal.outcome != action.outcome
            || terminal.completed < action.completed
            || !timing_matches
        {
            return Err(C1ArtifactReadError::InvalidResidue);
        }
        Ok(())
    }

    fn common_matches(expected: &CommonBinding, actual: &CommonBinding) -> bool {
        expected.family == actual.family
            && expected.execution_id == actual.execution_id
            && expected.plan_sha256 == actual.plan_sha256
            && expected.phase.as_str() == actual.phase.as_str()
    }

    fn valid_timestamp(value: u64) -> bool {
        value <= MAX_JSON_INTEGER
    }

    #[cfg(test)]
    mod test_hook {
        use std::sync::{Arc, Barrier, Mutex};
        use std::thread::{self, ThreadId};

        #[derive(Clone)]
        struct Hook {
            owner: ThreadId,
            target: String,
            reached: Arc<Barrier>,
            resume: Arc<Barrier>,
        }

        static HOOKS: Mutex<Vec<Hook>> = Mutex::new(Vec::new());

        pub(super) fn install(target: &str, reached: Arc<Barrier>, resume: Arc<Barrier>) {
            let owner = thread::current().id();
            let mut hooks = HOOKS.lock().expect("test hook mutex is available");
            hooks.retain(|hook| hook.owner != owner);
            hooks.push(Hook {
                owner,
                target: target.to_string(),
                reached,
                resume,
            });
        }

        pub(super) fn checkpoint(name: &str) {
            let hook = HOOKS
                .lock()
                .expect("test hook mutex is available")
                .iter()
                .find(|hook| hook.owner == thread::current().id() && hook.target == name)
                .cloned();
            if let Some(hook) = hook {
                hook.reached.wait();
                hook.resume.wait();
            }
        }

        pub(super) fn clear() {
            let owner = thread::current().id();
            HOOKS
                .lock()
                .expect("test hook mutex is available")
                .retain(|hook| hook.owner != owner);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;
        use std::io::Write as _;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::{symlink, PermissionsExt};
        use std::path::{Path, PathBuf};
        use std::sync::{Arc, Barrier};
        use std::thread;
        use std::time::Duration;

        const POLICY_SHA: &str = "70a8ddb1722e69009c05964df6f641b219f42c580180bd59dd8f9294f6533abb";
        const PLAN_SHA: &str = "1111111111111111111111111111111111111111111111111111111111111111";
        const STATIC_SHA: &str = "2222222222222222222222222222222222222222222222222222222222222222";
        const REQUEST_SHA: &str =
            "3333333333333333333333333333333333333333333333333333333333333333";
        const RESPONSE_SHA: &str =
            "4444444444444444444444444444444444444444444444444444444444444444";

        fn private_artifact_directory(root: &Path) -> PathBuf {
            let path = root.join("artifacts");
            fs::create_dir(&path).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            path.canonicalize().unwrap()
        }

        fn paused_classifier(
            directory: &Path,
            checkpoint: &str,
        ) -> (
            thread::JoinHandle<Result<C1OfflineState, C1ArtifactReadError>>,
            Arc<Barrier>,
            Arc<Barrier>,
        ) {
            let reached = Arc::new(Barrier::new(2));
            let resume = Arc::new(Barrier::new(2));
            let classifier_path = directory.to_path_buf();
            let target = checkpoint.to_string();
            let hook_reached = Arc::clone(&reached);
            let hook_resume = Arc::clone(&resume);
            let handle = thread::spawn(move || {
                test_hook::install(&target, hook_reached, hook_resume);
                let result = classify(&classifier_path);
                test_hook::clear();
                result
            });
            (handle, reached, resume)
        }

        fn document(family: &str, payload: Value) -> Vec<u8> {
            let mut bytes = serde_json::to_vec(&json!({
                "family": family,
                "schema_version": 1,
                "payload": payload,
            }))
            .unwrap();
            bytes.push(b'\n');
            bytes
        }

        fn write_pair(directory: &Path, primary_name: &str, bytes: &[u8]) -> String {
            let sidecar_name = format!("{primary_name}.sha256");
            let primary = directory.join(primary_name);
            let sidecar = directory.join(sidecar_name);
            fs::write(&primary, bytes).unwrap();
            fs::set_permissions(&primary, fs::Permissions::from_mode(0o600)).unwrap();
            let digest = format!("{:x}", Sha256::digest(bytes));
            fs::write(&sidecar, format!("{digest}\n")).unwrap();
            fs::set_permissions(&sidecar, fs::Permissions::from_mode(0o600)).unwrap();
            digest
        }

        fn intent(phase: &str) -> Vec<u8> {
            document(
                "native_correction_v1",
                json!({
                    "document_kind": "execution_intent",
                    "artifact_schema_version": 1,
                    "execution_id": "execution-1",
                    "plan_sha256": PLAN_SHA,
                    "phase": phase,
                    "sequence": 1,
                    "previous_artifact_sha256": null,
                    "phase_policy_id": "native-phase-artifact-policy-v1",
                    "phase_policy_sha256": POLICY_SHA,
                    "static_launch_template_sha256": STATIC_SHA,
                    "declared_started_unix_ms": 100,
                    "declared_terminal_unix_ms": 200,
                }),
            )
        }

        fn dispatch(phase: &str, intent_sha: &str) -> Vec<u8> {
            document(
                "native_correction_v1",
                json!({
                    "document_kind": "audit",
                    "artifact_kind": "mutation_dispatch",
                    "artifact_schema_version": 1,
                    "execution_id": "execution-1",
                    "plan_sha256": PLAN_SHA,
                    "phase": phase,
                    "sequence": 2,
                    "previous_artifact_sha256": intent_sha,
                    "ordinal": 1,
                    "request_sha256": REQUEST_SHA,
                    "created_unix_ms": 110,
                }),
            )
        }

        fn action(phase: &str, sequence: u64, previous: &str, started: u64) -> Vec<u8> {
            document(
                "native_correction_v1",
                json!({
                    "document_kind": "audit",
                    "artifact_kind": "phase_action_receipt",
                    "artifact_schema_version": 1,
                    "execution_id": "execution-1",
                    "plan_sha256": PLAN_SHA,
                    "phase": phase,
                    "sequence": sequence,
                    "previous_artifact_sha256": previous,
                    "ordinal": 1,
                    "request_sha256": REQUEST_SHA,
                    "response_projection_sha256": RESPONSE_SHA,
                    "outcome": "success",
                    "started_unix_ms": started,
                    "completed_unix_ms": started + 10,
                }),
            )
        }

        fn terminal(
            phase: &str,
            sequence: u64,
            intent_sha: &str,
            dispatch_sha: Option<&str>,
            action_sha: &str,
            completed: u64,
            timing_claim: &str,
        ) -> Vec<u8> {
            document(
                "native_correction_v1",
                json!({
                    "document_kind": "terminal_receipt",
                    "artifact_schema_version": 1,
                    "execution_id": "execution-1",
                    "plan_sha256": PLAN_SHA,
                    "phase": phase,
                    "sequence": sequence,
                    "previous_artifact_sha256": action_sha,
                    "intent_sha256": intent_sha,
                    "dispatch_sha256": dispatch_sha,
                    "action_sha256": action_sha,
                    "outcome": "success",
                    "timing_claim": timing_claim,
                    "completed_unix_ms": completed,
                }),
            )
        }

        fn insert_pair(
            entries: &mut BTreeMap<String, Vec<u8>>,
            primary_name: &str,
            bytes: Vec<u8>,
        ) -> String {
            let digest = format!("{:x}", Sha256::digest(&bytes));
            entries.insert(primary_name.to_string(), bytes);
            entries.insert(
                format!("{primary_name}.sha256"),
                format!("{digest}\n").into_bytes(),
            );
            digest
        }

        fn replace_payload_field(bytes: &[u8], key: &str, replacement: Value) -> Vec<u8> {
            let mut value: Value = serde_json::from_slice(bytes).unwrap();
            value
                .get_mut("payload")
                .and_then(Value::as_object_mut)
                .unwrap()
                .insert(key.to_string(), replacement);
            let mut bytes = serde_json::to_vec(&value).unwrap();
            bytes.push(b'\n');
            bytes
        }

        fn replace_outer_field(bytes: &[u8], key: &str, replacement: Value) -> Vec<u8> {
            let mut value: Value = serde_json::from_slice(bytes).unwrap();
            value
                .as_object_mut()
                .unwrap()
                .insert(key.to_string(), replacement);
            let mut bytes = serde_json::to_vec(&value).unwrap();
            bytes.push(b'\n');
            bytes
        }

        fn no_dispatch_action_entries(action_bytes: Vec<u8>) -> BTreeMap<String, Vec<u8>> {
            let mut entries = BTreeMap::new();
            let intent_sha = insert_pair(&mut entries, INTENT, intent("correction_retrieval"));
            let action_bytes =
                replace_payload_field(&action_bytes, "previous_artifact_sha256", json!(intent_sha));
            insert_pair(&mut entries, ACTION, action_bytes);
            entries
        }

        #[test]
        fn classifies_every_no_dispatch_prefix_and_timing_claim() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            assert_eq!(classify(&directory), Ok(C1OfflineState::Pristine));

            let intent_sha = write_pair(&directory, INTENT, &intent("correction_retrieval"));
            assert_eq!(classify(&directory), Ok(C1OfflineState::IntentOnlyFrozen));

            let action_sha = write_pair(
                &directory,
                ACTION,
                &action("correction_retrieval", 2, &intent_sha, 120),
            );
            assert_eq!(
                classify(&directory),
                Ok(C1OfflineState::IncompleteFrozenAfterAction)
            );

            write_pair(
                &directory,
                TERMINAL,
                &terminal(
                    "correction_retrieval",
                    3,
                    &intent_sha,
                    None,
                    &action_sha,
                    150,
                    "timely",
                ),
            );
            assert_eq!(
                classify(&directory),
                Ok(C1OfflineState::TerminalPairTimingUnproven)
            );

            write_pair(
                &directory,
                TERMINAL,
                &terminal(
                    "correction_retrieval",
                    3,
                    &intent_sha,
                    None,
                    &action_sha,
                    201,
                    "late",
                ),
            );
            assert_eq!(
                classify(&directory),
                Ok(C1OfflineState::TerminalPairClaimsLate)
            );
        }

        #[test]
        fn classifies_required_dispatch_prefix_and_chain() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            let intent_sha = write_pair(&directory, INTENT, &intent("correction_proposal"));
            let dispatch_sha = write_pair(
                &directory,
                DISPATCH,
                &dispatch("correction_proposal", &intent_sha),
            );
            assert_eq!(
                classify(&directory),
                Ok(C1OfflineState::MutationDispatchOutcomeUnknown)
            );
            write_pair(
                &directory,
                ACTION,
                &action("correction_proposal", 3, &dispatch_sha, 120),
            );
            assert_eq!(
                classify(&directory),
                Ok(C1OfflineState::IncompleteFrozenAfterAction)
            );
        }

        #[test]
        fn rejects_digest_forbidden_dispatch_and_non_prefix_sets() {
            let mut entries = BTreeMap::new();
            insert_pair(&mut entries, INTENT, intent("correction_retrieval"));
            entries.insert(
                INTENT_SIDECAR.to_string(),
                format!("{}\n", PLAN_SHA).into_bytes(),
            );
            assert_eq!(
                classify_entries(&entries),
                Err(C1ArtifactReadError::InvalidResidue)
            );

            let mut entries = BTreeMap::new();
            let intent_sha = insert_pair(&mut entries, INTENT, intent("correction_retrieval"));
            insert_pair(
                &mut entries,
                DISPATCH,
                dispatch("correction_retrieval", &intent_sha),
            );
            assert_eq!(
                classify_entries(&entries),
                Err(C1ArtifactReadError::InvalidResidue)
            );

            let mut action_only = BTreeMap::new();
            insert_pair(
                &mut action_only,
                ACTION,
                action("correction_retrieval", 2, PLAN_SHA, 120),
            );
            assert_eq!(
                classify_entries(&action_only),
                Err(C1ArtifactReadError::InvalidResidue)
            );
            let mut terminal_only = BTreeMap::new();
            insert_pair(
                &mut terminal_only,
                TERMINAL,
                terminal(
                    "correction_retrieval",
                    3,
                    PLAN_SHA,
                    None,
                    PLAN_SHA,
                    150,
                    "timely",
                ),
            );
            assert_eq!(
                classify_entries(&terminal_only),
                Err(C1ArtifactReadError::InvalidResidue)
            );
        }

        #[test]
        fn rejects_every_cross_binding_and_policy_mismatch() {
            let base_action = action("correction_retrieval", 2, PLAN_SHA, 120);
            for (field, replacement) in [
                ("execution_id", json!("execution-2")),
                ("plan_sha256", json!(STATIC_SHA)),
                ("phase", json!("correction_operator")),
                ("sequence", json!(3)),
                ("previous_artifact_sha256", json!(STATIC_SHA)),
            ] {
                let mut entries = no_dispatch_action_entries(replace_payload_field(
                    &base_action,
                    field,
                    replacement,
                ));
                if field == "previous_artifact_sha256" {
                    let bytes = entries.get(ACTION).unwrap().clone();
                    insert_pair(
                        &mut entries,
                        ACTION,
                        replace_payload_field(&bytes, field, json!(STATIC_SHA)),
                    );
                }
                assert_eq!(
                    classify_entries(&entries),
                    Err(C1ArtifactReadError::InvalidResidue),
                    "{field}"
                );
            }

            let mismatched_family =
                replace_outer_field(&base_action, "family", json!("native_stale_isolated_v1"));
            assert_eq!(
                classify_entries(&no_dispatch_action_entries(mismatched_family)),
                Err(C1ArtifactReadError::InvalidResidue)
            );

            for (field, replacement) in [
                ("phase_policy_id", json!("wrong-policy")),
                ("phase_policy_sha256", json!(PLAN_SHA)),
                ("artifact_schema_version", json!(2)),
                ("sequence", json!(2)),
                ("previous_artifact_sha256", json!(PLAN_SHA)),
            ] {
                let mut entries = BTreeMap::new();
                insert_pair(
                    &mut entries,
                    INTENT,
                    replace_payload_field(&intent("correction_retrieval"), field, replacement),
                );
                assert_eq!(
                    classify_entries(&entries),
                    Err(C1ArtifactReadError::InvalidResidue),
                    "{field}"
                );
            }

            let mut entries = BTreeMap::new();
            let intent_sha = insert_pair(&mut entries, INTENT, intent("correction_proposal"));
            let dispatch_sha = insert_pair(
                &mut entries,
                DISPATCH,
                dispatch("correction_proposal", &intent_sha),
            );
            insert_pair(
                &mut entries,
                ACTION,
                replace_payload_field(
                    &action("correction_proposal", 3, &dispatch_sha, 120),
                    "request_sha256",
                    json!(STATIC_SHA),
                ),
            );
            assert_eq!(
                classify_entries(&entries),
                Err(C1ArtifactReadError::InvalidResidue)
            );
        }

        #[test]
        fn enforces_chronology_outcome_and_timing_boundaries() {
            let mut entries = BTreeMap::new();
            let intent_sha = insert_pair(&mut entries, INTENT, intent("correction_proposal"));
            let dispatch_sha = insert_pair(
                &mut entries,
                DISPATCH,
                replace_payload_field(
                    &dispatch("correction_proposal", &intent_sha),
                    "created_unix_ms",
                    json!(100),
                ),
            );
            let action_sha = insert_pair(
                &mut entries,
                ACTION,
                replace_payload_field(
                    &action("correction_proposal", 3, &dispatch_sha, 100),
                    "completed_unix_ms",
                    json!(100),
                ),
            );
            insert_pair(
                &mut entries,
                TERMINAL,
                terminal(
                    "correction_proposal",
                    4,
                    &intent_sha,
                    Some(&dispatch_sha),
                    &action_sha,
                    100,
                    "timely",
                ),
            );
            assert_eq!(
                classify_entries(&entries),
                Ok(C1OfflineState::TerminalPairTimingUnproven)
            );

            for (field, replacement) in [
                ("started_unix_ms", json!(99)),
                ("completed_unix_ms", json!(119)),
            ] {
                let invalid = replace_payload_field(
                    &action("correction_retrieval", 2, PLAN_SHA, 120),
                    field,
                    replacement,
                );
                assert_eq!(
                    classify_entries(&no_dispatch_action_entries(invalid)),
                    Err(C1ArtifactReadError::InvalidResidue),
                    "{field}"
                );
            }

            for created in [99, 121] {
                let mut entries = BTreeMap::new();
                let intent_sha = insert_pair(&mut entries, INTENT, intent("correction_proposal"));
                let dispatch_sha = insert_pair(
                    &mut entries,
                    DISPATCH,
                    replace_payload_field(
                        &dispatch("correction_proposal", &intent_sha),
                        "created_unix_ms",
                        json!(created),
                    ),
                );
                insert_pair(
                    &mut entries,
                    ACTION,
                    action("correction_proposal", 3, &dispatch_sha, 120),
                );
                assert_eq!(
                    classify_entries(&entries),
                    Err(C1ArtifactReadError::InvalidResidue),
                    "created={created}"
                );
            }

            for (completed, claim) in [(201, "timely"), (150, "late"), (129, "timely")] {
                let mut entries = BTreeMap::new();
                let intent_sha = insert_pair(&mut entries, INTENT, intent("correction_retrieval"));
                let action_sha = insert_pair(
                    &mut entries,
                    ACTION,
                    action("correction_retrieval", 2, &intent_sha, 120),
                );
                let terminal_bytes = terminal(
                    "correction_retrieval",
                    3,
                    &intent_sha,
                    None,
                    &action_sha,
                    completed,
                    claim,
                );
                insert_pair(&mut entries, TERMINAL, terminal_bytes);
                assert_eq!(
                    classify_entries(&entries),
                    Err(C1ArtifactReadError::InvalidResidue),
                    "completed={completed}, claim={claim}"
                );
            }

            let mut entries = BTreeMap::new();
            let intent_sha = insert_pair(&mut entries, INTENT, intent("correction_retrieval"));
            let action_sha = insert_pair(
                &mut entries,
                ACTION,
                action("correction_retrieval", 2, &intent_sha, 120),
            );
            insert_pair(
                &mut entries,
                TERMINAL,
                replace_payload_field(
                    &terminal(
                        "correction_retrieval",
                        3,
                        &intent_sha,
                        None,
                        &action_sha,
                        150,
                        "timely",
                    ),
                    "outcome",
                    json!("failure"),
                ),
            );
            assert_eq!(
                classify_entries(&entries),
                Err(C1ArtifactReadError::InvalidResidue)
            );
        }

        #[test]
        fn rejects_malformed_duplicate_and_unknown_envelopes() {
            let malformed = b"{\"family\":\n".to_vec();
            let duplicate = concat!(
                "{\"family\":\"native_correction_v1\",",
                "\"family\":\"native_correction_v1\",",
                "\"schema_version\":1,\"payload\":{}}\n"
            )
            .as_bytes()
            .to_vec();
            let unknown =
                replace_outer_field(&intent("correction_retrieval"), "unexpected", json!(true));
            for bytes in [malformed, duplicate, unknown] {
                let mut entries = BTreeMap::new();
                insert_pair(&mut entries, INTENT, bytes);
                assert_eq!(
                    classify_entries(&entries),
                    Err(C1ArtifactReadError::InvalidResidue)
                );
            }

            let unknown_payload =
                replace_payload_field(&intent("correction_retrieval"), "unexpected", json!(true));
            let mut entries = BTreeMap::new();
            insert_pair(&mut entries, INTENT, unknown_payload);
            assert_eq!(
                classify_entries(&entries),
                Err(C1ArtifactReadError::InvalidResidue)
            );
        }

        #[test]
        fn enforces_entry_cap_and_exact_primary_and_sidecar_limits() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            for index in 0..=MAX_ENTRIES {
                let path = directory.join(format!("entry-{index}"));
                fs::write(&path, b"x").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            }
            assert_eq!(
                classify(&directory),
                Err(C1ArtifactReadError::InvalidResidue)
            );

            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            let mut maximum_primary = intent("correction_retrieval");
            maximum_primary.splice(
                maximum_primary.len() - 2..maximum_primary.len() - 2,
                std::iter::repeat(b' ').take(MAX_PRIMARY_BYTES as usize - maximum_primary.len()),
            );
            assert_eq!(maximum_primary.len(), MAX_PRIMARY_BYTES as usize);
            write_pair(&directory, INTENT, &maximum_primary);
            assert_eq!(classify(&directory), Ok(C1OfflineState::IntentOnlyFrozen));

            maximum_primary.insert(maximum_primary.len() - 2, b' ');
            write_pair(&directory, INTENT, &maximum_primary);
            assert_eq!(
                classify(&directory),
                Err(C1ArtifactReadError::InvalidResidue)
            );

            for sidecar_length in [64, 66] {
                let root = tempfile::tempdir().unwrap();
                let directory = private_artifact_directory(root.path());
                write_pair(&directory, INTENT, &intent("correction_retrieval"));
                let sidecar = directory.join(INTENT_SIDECAR);
                fs::write(&sidecar, vec![b'a'; sidecar_length]).unwrap();
                fs::set_permissions(&sidecar, fs::Permissions::from_mode(0o600)).unwrap();
                assert_eq!(
                    classify(&directory),
                    Err(C1ArtifactReadError::InvalidResidue),
                    "sidecar length {sidecar_length}"
                );
            }
        }

        #[test]
        fn rejects_unsafe_directory_paths_modes_and_acls() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).unwrap();
            assert_eq!(
                classify(&directory),
                Err(C1ArtifactReadError::UnsafeDirectory)
            );

            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            let link = root.path().join("artifact-link");
            symlink(&directory, &link).unwrap();
            assert_eq!(classify(&link), Err(C1ArtifactReadError::UnsafeDirectory));

            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            let status = std::process::Command::new("/bin/chmod")
                .args(["+a", "everyone allow write", directory.to_str().unwrap()])
                .status()
                .unwrap();
            assert!(status.success());
            assert_eq!(
                classify(&directory),
                Err(C1ArtifactReadError::UnsafeDirectory)
            );

            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let primary = directory.join(INTENT);
            let status = std::process::Command::new("/bin/chmod")
                .args(["+a", "everyone allow write", primary.to_str().unwrap()])
                .status()
                .unwrap();
            assert!(status.success());
            assert_eq!(classify(&directory), Err(C1ArtifactReadError::UnsafeEntry));
        }

        #[test]
        fn errors_are_categorical_and_production_region_is_capability_free() {
            for (error, spelling) in [
                (
                    C1ArtifactReadError::UnsupportedPlatform,
                    "UnsupportedPlatform",
                ),
                (C1ArtifactReadError::Unreadable, "Unreadable"),
                (C1ArtifactReadError::UnsafeDirectory, "UnsafeDirectory"),
                (C1ArtifactReadError::UnsafeEntry, "UnsafeEntry"),
                (C1ArtifactReadError::IdentityChanged, "IdentityChanged"),
                (C1ArtifactReadError::InvalidResidue, "InvalidResidue"),
            ] {
                assert_eq!(format!("{error:?}"), spelling);
            }

            let source = include_str!("native_successor_artifact.rs");
            let production = source
                .split("    #[cfg(test)]\n    mod test_hook")
                .next()
                .expect("production source region exists");
            assert_eq!(production.matches("ENTRY_OPEN_FLAGS").count(), 2);
            assert!(production.contains("libc::O_NONBLOCK"));
            let compact_source: String = production
                .chars()
                .filter(|character| !character.is_ascii_whitespace())
                .collect();
            assert!(compact_source.contains(concat!(
                "libc::openat(self.file.as_raw_fd(),",
                "name_c.as_ptr(),ENTRY_OPEN_FLAGS)"
            )));
            assert_eq!(
                compact_source
                    .matches("fnrevalidate_entry_boundary(")
                    .count(),
                1
            );
            assert!(compact_source.contains(concat!(
                "letseek_result=self.file.seek(SeekFrom::Start(0))",
                ".map_err(|_|C1ArtifactReadError::Unreadable);",
                "letseek_boundary=revalidate_entry_boundary(&self.file,directory_fd,",
                "&self.name,effective_uid,self.identity,);seek_boundary?;seek_result?;"
            )));
            for forbidden in [
                "native_successor_core",
                "native_execution",
                "native_runner",
                "native_pilot",
                "native_isolation",
                "std::process",
                "std::env",
                "std::net",
                "tokio",
                "reqwest",
                "std::thread",
                "std::sync",
                "Mutex",
                "Barrier",
                "create_dir",
                "fs::write",
                "fs::rename",
                "remove_file",
                "set_permissions",
                "fs::symlink(",
                "hard_link",
            ] {
                assert!(
                    !production.contains(forbidden),
                    "production source contains forbidden capability: {forbidden}"
                );
            }
        }

        #[test]
        fn rejects_partial_extra_and_unsafe_entries_categorically() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            fs::write(directory.join(INTENT), intent("correction_retrieval")).unwrap();
            fs::set_permissions(directory.join(INTENT), fs::Permissions::from_mode(0o600)).unwrap();
            assert_eq!(
                classify(&directory),
                Err(C1ArtifactReadError::InvalidResidue)
            );

            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let extra = directory.join("extra");
            fs::write(&extra, b"fixture").unwrap();
            fs::set_permissions(&extra, fs::Permissions::from_mode(0o600)).unwrap();
            assert_eq!(
                classify(&directory),
                Err(C1ArtifactReadError::InvalidResidue)
            );

            fs::remove_file(extra).unwrap();
            fs::set_permissions(directory.join(INTENT), fs::Permissions::from_mode(0o644)).unwrap();
            assert_eq!(classify(&directory), Err(C1ArtifactReadError::UnsafeEntry));
        }

        #[test]
        fn rejects_symlinks_and_hardlinks() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let sidecar = directory.join(INTENT_SIDECAR);
            let moved = root.path().join("sidecar-target");
            fs::rename(&sidecar, &moved).unwrap();
            symlink(&moved, &sidecar).unwrap();
            assert_eq!(classify(&directory), Err(C1ArtifactReadError::UnsafeEntry));

            fs::remove_file(&sidecar).unwrap();
            fs::rename(&moved, &sidecar).unwrap();
            let hardlink = root.path().join("hardlink-target");
            fs::hard_link(directory.join(INTENT), hardlink).unwrap();
            assert_eq!(classify(&directory), Err(C1ArtifactReadError::UnsafeEntry));
        }

        #[test]
        fn entry_open_flags_are_exact_and_fifo_substitution_is_nonreading() {
            assert_eq!(
                ENTRY_OPEN_FLAGS,
                libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC
            );

            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) =
                paused_classifier(&directory, &format!("before-open:{INTENT}"));

            reached.wait();
            let primary = directory.join(INTENT);
            fs::rename(&primary, root.path().join("intent-original")).unwrap();
            let fifo_path = CString::new(primary.as_os_str().as_bytes()).unwrap();
            assert_eq!(unsafe { libc::mkfifo(fifo_path.as_ptr(), 0o600) }, 0);
            fs::set_permissions(&primary, fs::Permissions::from_mode(0o600)).unwrap();
            let mut keeper = OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
                .open(&primary)
                .unwrap();
            keeper.write_all(b"X").unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
            let mut sentinel = [0_u8; 1];
            keeper.read_exact(&mut sentinel).unwrap();
            assert_eq!(sentinel, *b"X");
        }

        #[test]
        fn pre_open_symlink_substitution_is_unsafe_without_following() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) =
                paused_classifier(&directory, &format!("before-open:{INTENT}"));

            reached.wait();
            let primary = directory.join(INTENT);
            let moved = root.path().join("intent-original");
            fs::rename(&primary, &moved).unwrap();
            symlink(&moved, &primary).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn pre_open_directory_mode_zero_reports_unsafe_directory() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) =
                paused_classifier(&directory, &format!("before-open:{INTENT}"));

            reached.wait();
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o000)).unwrap();
            let moved = root.path().join("artifacts-moved");
            fs::rename(&directory, &moved).unwrap();
            resume.wait();
            let result = handle.join().unwrap();
            fs::set_permissions(&moved, fs::Permissions::from_mode(0o700)).unwrap();

            assert_eq!(result, Err(C1ArtifactReadError::UnsafeDirectory));
        }

        #[test]
        fn post_first_sweep_directory_mode_zero_reports_unsafe_directory() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) = paused_classifier(&directory, "after-first-sweep");

            reached.wait();
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o000)).unwrap();
            let moved = root.path().join("artifacts-moved");
            fs::rename(&directory, &moved).unwrap();
            resume.wait();
            let result = handle.join().unwrap();
            fs::set_permissions(&moved, fs::Permissions::from_mode(0o700)).unwrap();

            assert_eq!(result, Err(C1ArtifactReadError::UnsafeDirectory));
        }

        #[test]
        fn first_sweep_first_read_unsafe_mode_precedes_short_read_error() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) =
                paused_classifier(&directory, &format!("before-first-sweep-read:{INTENT}"));

            reached.wait();
            let primary = directory.join(INTENT);
            fs::write(&primary, b"x").unwrap();
            fs::set_permissions(&primary, fs::Permissions::from_mode(0o644)).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn first_sweep_second_read_unsafe_mode_precedes_short_read_error() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) = paused_classifier(
                &directory,
                &format!("before-second-first-sweep-read:{INTENT}"),
            );

            reached.wait();
            let primary = directory.join(INTENT);
            fs::write(&primary, b"x").unwrap();
            fs::set_permissions(&primary, fs::Permissions::from_mode(0o644)).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn first_sweep_post_read_handle_safety_precedes_missing_path() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) = paused_classifier(&directory, INTENT);

            reached.wait();
            let primary = directory.join(INTENT);
            let moved = root.path().join("intent-moved-unsafe");
            fs::rename(&primary, &moved).unwrap();
            fs::set_permissions(&moved, fs::Permissions::from_mode(0o644)).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn second_sweep_post_read_handle_safety_precedes_missing_path() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) =
                paused_classifier(&directory, &format!("before-second-sweep-read:{INTENT}"));

            reached.wait();
            let primary = directory.join(INTENT);
            let moved = root.path().join("intent-moved-unsafe");
            fs::rename(&primary, &moved).unwrap();
            fs::set_permissions(&moved, fs::Permissions::from_mode(0o644)).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn post_reread_unsafe_mode_precedes_byte_mismatch() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            let original = intent("correction_retrieval");
            write_pair(&directory, INTENT, &original);
            let (handle, reached, resume) =
                paused_classifier(&directory, &format!("before-second-sweep-read:{INTENT}"));

            reached.wait();
            let mut replacement = original;
            replacement[0] = b'[';
            let primary = directory.join(INTENT);
            fs::write(&primary, replacement).unwrap();
            fs::set_permissions(&primary, fs::Permissions::from_mode(0o644)).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn post_reread_unsafe_mode_precedes_short_read_error() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) =
                paused_classifier(&directory, &format!("before-second-sweep-read:{INTENT}"));

            reached.wait();
            let primary = directory.join(INTENT);
            fs::write(&primary, b"x").unwrap();
            fs::set_permissions(&primary, fs::Permissions::from_mode(0o644)).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn final_sweep_detects_same_length_primary_overwrite() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            let original = intent("correction_retrieval");
            write_pair(&directory, INTENT, &original);
            let (handle, reached, resume) = paused_classifier(&directory, "after-first-sweep");

            reached.wait();
            thread::sleep(Duration::from_millis(2));
            let mut replacement = original.clone();
            replacement[0] = b'[';
            fs::write(directory.join(INTENT), replacement).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::IdentityChanged)
            );
        }

        #[test]
        fn final_sweep_reports_unsafe_sidecar_mode_before_identity_change() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) = paused_classifier(&directory, "after-first-sweep");

            reached.wait();
            fs::set_permissions(
                directory.join(INTENT_SIDECAR),
                fs::Permissions::from_mode(0o644),
            )
            .unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn final_sweep_reports_unsafe_entry_acl_before_identity_change() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) = paused_classifier(&directory, "after-first-sweep");

            reached.wait();
            let status = std::process::Command::new("/bin/chmod")
                .args([
                    "+a",
                    "everyone allow write",
                    directory.join(INTENT).to_str().unwrap(),
                ])
                .status()
                .unwrap();
            assert!(status.success());
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeEntry)
            );
        }

        #[test]
        fn final_sweep_detects_overwrite_then_restore_through_ctime() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            let original = intent("correction_retrieval");
            write_pair(&directory, INTENT, &original);
            let primary = directory.join(INTENT);
            let original_metadata = fs::metadata(&primary).unwrap();
            let (handle, reached, resume) = paused_classifier(&directory, "after-first-sweep");

            reached.wait();
            thread::sleep(Duration::from_millis(2));
            let mut replacement = original.clone();
            replacement[0] = b'[';
            fs::write(&primary, replacement).unwrap();
            fs::write(&primary, &original).unwrap();
            let primary_c = CString::new(primary.as_os_str().as_bytes()).unwrap();
            let times = [
                libc::timespec {
                    tv_sec: 0,
                    tv_nsec: libc::UTIME_OMIT,
                },
                libc::timespec {
                    tv_sec: original_metadata.mtime(),
                    tv_nsec: original_metadata.mtime_nsec(),
                },
            ];
            assert_eq!(
                unsafe {
                    libc::utimensat(
                        libc::AT_FDCWD,
                        primary_c.as_ptr(),
                        times.as_ptr(),
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                },
                0
            );
            let restored_metadata = fs::metadata(&primary).unwrap();
            assert_eq!(restored_metadata.size(), original_metadata.size());
            assert_eq!(
                (restored_metadata.mtime(), restored_metadata.mtime_nsec()),
                (original_metadata.mtime(), original_metadata.mtime_nsec())
            );
            assert_ne!(
                (restored_metadata.ctime(), restored_metadata.ctime_nsec()),
                (original_metadata.ctime(), original_metadata.ctime_nsec())
            );
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::IdentityChanged)
            );
        }

        #[test]
        fn final_sweep_reports_unsafe_directory_mode_before_identity_change() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) = paused_classifier(&directory, "after-first-sweep");

            reached.wait();
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).unwrap();
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeDirectory)
            );
        }

        #[test]
        fn final_sweep_reports_unsafe_directory_acl_before_identity_change() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let (handle, reached, resume) = paused_classifier(&directory, "after-first-sweep");

            reached.wait();
            let status = std::process::Command::new("/bin/chmod")
                .args(["+a", "everyone allow write", directory.to_str().unwrap()])
                .status()
                .unwrap();
            assert!(status.success());
            resume.wait();

            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::UnsafeDirectory)
            );
        }

        #[test]
        fn retained_entry_descriptor_detects_path_replacement() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let reached = Arc::new(Barrier::new(2));
            let resume = Arc::new(Barrier::new(2));
            let classifier_path = directory.clone();
            let hook_reached = Arc::clone(&reached);
            let hook_resume = Arc::clone(&resume);
            let handle = thread::spawn(move || {
                test_hook::install(INTENT, hook_reached, hook_resume);
                let result = classify(&classifier_path);
                test_hook::clear();
                result
            });

            reached.wait();
            let original = directory.join(INTENT);
            fs::rename(&original, directory.join("intent-moved")).unwrap();
            fs::write(&original, intent("correction_retrieval")).unwrap();
            fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();
            resume.wait();
            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::IdentityChanged)
            );
        }

        #[test]
        fn retained_directory_descriptor_detects_path_replacement() {
            let root = tempfile::tempdir().unwrap();
            let directory = private_artifact_directory(root.path());
            write_pair(&directory, INTENT, &intent("correction_retrieval"));
            let reached = Arc::new(Barrier::new(2));
            let resume = Arc::new(Barrier::new(2));
            let classifier_path = directory.clone();
            let hook_reached = Arc::clone(&reached);
            let hook_resume = Arc::clone(&resume);
            let handle = thread::spawn(move || {
                test_hook::install(INTENT, hook_reached, hook_resume);
                let result = classify(&classifier_path);
                test_hook::clear();
                result
            });

            reached.wait();
            fs::rename(&directory, root.path().join("artifacts-moved")).unwrap();
            fs::create_dir(&directory).unwrap();
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
            resume.wait();
            assert_eq!(
                handle.join().unwrap(),
                Err(C1ArtifactReadError::IdentityChanged)
            );
        }
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod non_macos_tests {
    use super::*;

    #[test]
    fn unsupported_platform_returns_before_path_io() {
        let path = Path::new("/a/path/that/must/not/be/inspected");
        assert_eq!(
            classify_c1_artifact_directory(path),
            Err(C1ArtifactReadError::UnsupportedPlatform)
        );
    }
}
