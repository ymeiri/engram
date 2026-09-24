#!/usr/bin/env python3
"""Closed, subprocess-free provisioner for the frozen C2B2a build."""

from __future__ import annotations

import ctypes
import errno
import hashlib
import json
import os
import re
import stat
import sys
import tarfile
import tomllib
import unicodedata
from dataclasses import dataclass
from typing import BinaryIO, Mapping, Sequence


U64_MAX = (1 << 64) - 1
MANIFEST_BYTES_MAX = 1_048_576
MANIFEST_ROWS_MAX = 1_024
ARCHIVE_BYTES_MAX = 268_435_456
ARCHIVES_BYTES_MAX = 4_294_967_296
ARCHIVE_MEMBERS_MAX = 200_000
ARCHIVES_MEMBERS_MAX = 2_000_000
REALIZED_ENTRIES_MAX = 2_100_000
FILE_BYTES_MAX = 536_870_912
ARCHIVE_EXPANDED_MAX = 4_294_967_296
ARCHIVES_EXPANDED_MAX = 17_179_869_184
PATH_COMPONENTS_MAX = 128
PATH_BYTES_MAX = 4_096
READ_CHUNK = 1 << 20

REPOSITORY_PAYLOAD = (
    "/Users/yuval.meiri/projects/engram/engram-eval/native-c2b2a-payload"
)
REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
ROOT_PACKAGE = ("engram-native-c2b2a-payload", "0.0.0")
MANIFEST_SCHEMA = b"schema\tc2b2a-archive-manifest-v1\n"
ROOT_CONFIG_BYTES = (
    b'[source.crates-io]\n'
    b'replace-with = "c2b2a-archive-source"\n'
    b'\n'
    b'[source.c2b2a-archive-source]\n'
    b'directory = "cargo-source"\n'
    b'\n'
    b'[net]\n'
    b'offline = true\n'
)
PAYLOAD_CONFIG_PRE_BYTES = (
    b'[build]\n'
    b'target-dir = "/private/tmp/engram-stage-b-20260906"\n'
    b'\n'
    b'[target.aarch64-unknown-linux-musl]\n'
    b'linker = "/opt/homebrew/bin/aarch64-linux-musl-gcc"\n'
    b'ar = "/opt/homebrew/bin/aarch64-linux-musl-ar"\n'
    b'rustflags = [\n'
    b'    "--cfg",\n'
    b'    "getrandom_backend=\\"linux_getrandom\\"",\n'
    b'    "-C",\n'
    b'    "relocation-model=pie",\n'
    b'    "-C",\n'
    b'    "link-arg=-static-pie",\n'
    b']\n'
)
PAYLOAD_CONFIG_ANCHOR = b'    "-C",\n    "link-arg=-static-pie",\n'
PAYLOAD_CONFIG_INSERTION = b'    "-C",\n    "link-arg=-Wl,--build-id=sha1",\n'
if (
    len(PAYLOAD_CONFIG_PRE_BYTES) != 346
    or hashlib.sha256(PAYLOAD_CONFIG_PRE_BYTES).hexdigest()
    != "cbe42bd9dbe49e8dd350166544e3cfbdc372a4a9abbd196d41f55b55d0f341ec"
    or PAYLOAD_CONFIG_PRE_BYTES.count(PAYLOAD_CONFIG_ANCHOR) != 1
):
    raise RuntimeError("frozen payload configuration preimage is inconsistent")
PAYLOAD_CONFIG_BYTES = PAYLOAD_CONFIG_PRE_BYTES.replace(
    PAYLOAD_CONFIG_ANCHOR,
    PAYLOAD_CONFIG_ANCHOR + PAYLOAD_CONFIG_INSERTION,
)
if (
    PAYLOAD_CONFIG_BYTES.count(PAYLOAD_CONFIG_ANCHOR + PAYLOAD_CONFIG_INSERTION) != 1
    or PAYLOAD_CONFIG_BYTES.replace(
        PAYLOAD_CONFIG_ANCHOR + PAYLOAD_CONFIG_INSERTION,
        PAYLOAD_CONFIG_ANCHOR,
        1,
    )
    != PAYLOAD_CONFIG_PRE_BYTES
):
    raise RuntimeError("frozen payload configuration delta is inconsistent")

CARGO_TOML_PRE_BYTES = b"""[package]
name = "engram-native-c2b2a-payload"
version = "0.0.0"
edition = "2021"
rust-version = "1.93"
publish = false
autolib = false
autobins = false
autoexamples = false
autotests = false
autobenches = false

[workspace]

[features]
default = []
supervisor = []
collector = ["dep:surrealdb-core", "dep:tokio"]

[dependencies]
getrandom = { version = "=0.3.4", default-features = false }
libc = { version = "=0.2.180", default-features = false }
sha2 = { version = "=0.10.9", default-features = false }

[dependencies.surrealdb-core]
version = "=2.6.0"
default-features = false
features = ["kv-rocksdb"]
optional = true

[dependencies.tokio]
version = "=1.49.0"
default-features = false
features = ["rt", "sync", "time"]
optional = true

[lib]
name = "engram_native_c2b2a_payload"
path = "src/lib.rs"

[[bin]]
name = "supervisor"
path = "src/bin/supervisor.rs"
required-features = ["supervisor"]

[[bin]]
name = "collector"
path = "src/bin/collector.rs"
required-features = ["collector"]

[[test]]
name = "contract"
path = "tests/contract.rs"

[[test]]
name = "protocol"
path = "tests/protocol.rs"

[[test]]
name = "rocks"
path = "tests/rocks.rs"
required-features = ["collector"]

[[test]]
name = "seccomp"
path = "tests/seccomp.rs"
"""
CARGO_TOML_COLLECTOR_PRE = b'collector = ["dep:surrealdb-core", "dep:tokio"]\n'
CARGO_TOML_COLLECTOR_FINAL = (
    b'collector = [\n'
    b'    "dep:getrandom02",\n'
    b'    "dep:surrealdb-core",\n'
    b'    "dep:tokio",\n'
    b']\n'
)
CARGO_TOML_ALIAS = (
    b'[dependencies.getrandom02]\n'
    b'package = "getrandom"\n'
    b'version = "=0.2.17"\n'
    b'default-features = false\n'
    b'features = ["linux_disable_fallback"]\n'
    b'optional = true\n\n'
)
CARGO_TOML_ALIAS_ANCHOR = (
    b'sha2 = { version = "=0.10.9", default-features = false }\n\n'
    b'[dependencies.surrealdb-core]\n'
)
if (
    len(CARGO_TOML_PRE_BYTES) != 1_237
    or hashlib.sha256(CARGO_TOML_PRE_BYTES).hexdigest()
    != "3b28236b23a2b1116bcbbd696d9d6c0c78a707b6a2de791933c73277e5fb7ff8"
    or CARGO_TOML_PRE_BYTES.count(CARGO_TOML_COLLECTOR_PRE) != 1
    or CARGO_TOML_PRE_BYTES.count(CARGO_TOML_ALIAS_ANCHOR) != 1
):
    raise RuntimeError("frozen Cargo.toml preimage is inconsistent")
CARGO_TOML_BYTES = CARGO_TOML_PRE_BYTES.replace(
    CARGO_TOML_COLLECTOR_PRE,
    CARGO_TOML_COLLECTOR_FINAL,
).replace(
    CARGO_TOML_ALIAS_ANCHOR,
    CARGO_TOML_ALIAS_ANCHOR.split(b"[dependencies.surrealdb-core]\n", 1)[0]
    + CARGO_TOML_ALIAS
    + b"[dependencies.surrealdb-core]\n",
)
if (
    CARGO_TOML_BYTES.count(CARGO_TOML_COLLECTOR_FINAL) != 1
    or CARGO_TOML_BYTES.count(CARGO_TOML_ALIAS) != 1
    or CARGO_TOML_BYTES.replace(
        CARGO_TOML_COLLECTOR_FINAL,
        CARGO_TOML_COLLECTOR_PRE,
        1,
    ).replace(
        CARGO_TOML_ALIAS,
        b"",
        1,
    )
    != CARGO_TOML_PRE_BYTES
):
    raise RuntimeError("frozen Cargo.toml delta is inconsistent")

PATCH_SHA256 = "c82a6d05411362963eb8049cc6a0ecb563562bd1d35e14bfe892f39db936fcfb"
PATCH_BYTES = 1_565
PATCH_LINES = 64
ROCKS_NAME = "surrealdb-librocksdb-sys"
ROCKS_VERSION = "0.17.3+10.6.2"
ROCKS_ARCHIVE_SHA256 = "db194f1cf601bb6f2d0f4cbf0931bc3e5a602bac41ef2e9a87eccdfb28b7fed2"
ROCKS_FILES = {
    "rocksdb/env/unique_id_gen.cc": (
        "c689cd2a10dfe02bfb94f0b92d8730ee158a8a8ae7d5066beaf1facbc1232310",
        "44fcb9e1535c23ed9f0017e25be197fa0c365bf7a8a340aafa540526e485a4a5",
    ),
    "rocksdb/port/port_posix.cc": (
        "a34341c2e48b59a21037709c5cd34382991c4684a1db4f39e1ce73697ab2b62f",
        "70daf26d27666616456679eb63fecdb5f703606b4583da6af00e53e8b9c29bec",
    ),
}

BASELINE_DIRS = {
    ".cargo",
    "patches",
    "src",
    "src/bin",
    "tests",
}
BASELINE_FILES = {
    "rust-toolchain.toml": (
        127,
        "d5d0e289ce24dcad9cbe5620576217d1f57c8f5248cc14001a249816534ec8ab",
    ),
    "src/bin/collector.rs": (
        1180,
        "ec5ff01fc4ab16a334fc5f30b53741971c817ecf58a54e4b5fddf34058a1a88e",
    ),
    "src/bin/supervisor.rs": (
        1301,
        "4ceca71496337fa07565ef3df3c2b628dbb3e09e9a36f26266dc8183ad1aeff6",
    ),
    "src/contract.rs": (45528, "0c9ead1aefc00a722819481c23bf33fa064df7bfcff641ae45b96692ef4dabe8"),
    "src/fixtures.rs": (38934, "cdf143a561274512c510e0734d7f65f7d99c229e2a42978e5a8dafce512eece2"),
    "src/lib.rs": (181, "7b02babd0aa1f881fcfb9642196aafe6ed1181a63bb2bf8d272c18750153cac9"),
    "src/protocol.rs": (97347, "cbd747e0d35ccdcfc78972eb804fc8700c3076d8935efc57f949e651e39786c7"),
    "src/rocks.rs": (12054, "499b3986e4439c994cccab8dc4e43af636708853dddc8ad7062f3c236155101e"),
    "src/seccomp.rs": (29180, "e11d04fb371860bc3d40991ec8a9f48c9702878c72531d9b3d71adba49aab251"),
    "tests/protocol.rs": (
        33247,
        "96ef921788663ace466666cb2ad42b1f74d214631e3e6ecb0c1bdd6680253dfc",
    ),
    "tests/rocks.rs": (18649, "791290945222d9b567d83ddf1886c0a56bedded1f7852a31a03890a1fb2f5e09"),
    "tests/seccomp.rs": (13305, "80a7e604882f13204b118716eb094af664d99eec75567b1f5ed335719f4241e5"),
}
CHANGED_FILES = {
    ".cargo/config.toml",
    "Cargo.lock",
    "Cargo.toml",
    "patches/surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch",
    "tests/contract.rs",
}
SUPPORT_FILES = {
    "build-support/archive-manifest-v1.tsv",
    "build-support/controller.py",
    "build-support/entropy-failure-probe.cc",
    "build-support/entropy-failure-probe.rs",
    "build-support/provision.py",
    "build-support/root-cargo-config.toml",
}
CONTRACT_PRE_SIZE = 21_846
CONTRACT_PRE_SHA256 = "90a66276fc5109bb358f368a8dd271d603549d8a7ee8036b5f8c4129fb4a9af4"
CONTRACT_DIRECT_PRE = (
    b'    assert_eq!(\n'
    b'        direct_dependencies,\n'
    b'        ["getrandom", "libc", "sha2", "surrealdb-core", "tokio"]\n'
    b'    );\n'
)
CONTRACT_DIRECT_FINAL = (
    b'    assert_eq!(\n'
    b'        direct_dependencies,\n'
    b'        [\n'
    b'            "getrandom",\n'
    b'            "libc",\n'
    b'            "sha2",\n'
    b'            "getrandom02",\n'
    b'            "surrealdb-core",\n'
    b'            "tokio",\n'
    b'        ]\n'
    b'    );\n'
)
CONTRACT_MANIFEST_ASSERTIONS = b"".join(
    (
        b'    assert!(STANDALONE_MANIFEST.contains(\n',
        b'        "collector = [\\n\\',
        b'\n',
        b'    \\"dep:getrandom02\\",\\n\\',
        b'\n',
        b'    \\"dep:surrealdb-core\\",\\n\\',
        b'\n',
        b'    \\"dep:tokio\\",\\n\\',
        b'\n',
        b']"\n',
        b'    ));\n',
        b'    assert!(STANDALONE_MANIFEST.contains(\n',
        b'        "[dependencies.getrandom02]\\n\\',
        b'\n',
        b'package = \\"getrandom\\"\\n\\',
        b'\n',
        b'version = \\"=0.2.17\\"\\n\\',
        b'\n',
        b'default-features = false\\n\\',
        b'\n',
        b'features = [\\"linux_disable_fallback\\"]\\n\\',
        b'\n',
        b'optional = true"\n',
        b'    ));\n',
    )
)
CONTRACT_LOCK_TUPLE = (
    b'        (\n'
    b'            "getrandom",\n'
    b'            "0.2.17",\n'
    b'            "ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0",\n'
    b'        ),\n'
)
CONTRACT_LOCK_PREFIX = b'    for (name, version, checksum) in [\n'
CONTRACT_LOCK_FIRST_TUPLE = (
    b'        (\n'
    b'            "surrealdb-core",\n'
)
LOCK_ROOT_PRE = (
    b'[[package]]\n'
    b'name = "engram-native-c2b2a-payload"\n'
    b'version = "0.0.0"\n'
    b'dependencies = [\n'
    b' "getrandom 0.3.4",\n'
    b' "libc",\n'
    b' "sha2",\n'
    b' "surrealdb-core",\n'
    b' "tokio",\n'
    b']\n'
)
LOCK_ROOT_FINAL = LOCK_ROOT_PRE.replace(
    b'dependencies = [\n',
    b'dependencies = [\n "getrandom 0.2.17",\n',
)
if (
    LOCK_ROOT_FINAL.count(b' "getrandom 0.2.17",\n') != 1
    or LOCK_ROOT_FINAL.replace(b' "getrandom 0.2.17",\n', b"", 1) != LOCK_ROOT_PRE
):
    raise RuntimeError("frozen Cargo.lock root-edge delta is inconsistent")

if sys.platform != "darwin":
    raise RuntimeError("the frozen provisioner requires the reviewed Darwin host")
try:
    O_CLOEXEC = os.O_CLOEXEC
    O_NOFOLLOW = os.O_NOFOLLOW
    O_DIRECTORY = os.O_DIRECTORY
except AttributeError as exc:
    raise RuntimeError("required Darwin no-follow open flags are unavailable") from exc
if not O_CLOEXEC or not O_NOFOLLOW or not O_DIRECTORY:
    raise RuntimeError("required Darwin no-follow open flags are invalid")
ACL_TYPE_EXTENDED = 0x00000100
ACL_FIRST_ENTRY = 0
ACL_NEXT_ENTRY = -1
XATTR_NAME_LIST_MAX = 69
XATTR_VALUE_MAX = 61
SYSTEM_XATTR_NAME_LIST_MAX = 268_435_456
SYSTEM_XATTR_VALUE_MAX = 16_777_216
SYSTEM_XATTR_VALUES_TOTAL_MAX = 1_073_741_824
SYSTEM_XATTR_COUNT_MAX = 1_048_576
XATTR_SIDECAR_ROWS_MAX = 1_048_576
XATTR_SIDECAR_BYTES_MAX = 4_294_967_296
ACL_EXTERNAL_MAX = 1_048_576
STATFS_BYTES = 2_168
MNT_IGNORE_OWNERSHIP = 0x00200000

XATTR_PROFILE_ZERO = "zero"
XATTR_PROFILE_FRESH = "provenance-fresh"
XATTR_PROFILE_ANCESTOR = "provenance-ancestor"
XATTR_PROFILE_CARGO = "provenance-plus-backup"
PROVENANCE_NAME = b"com.apple.provenance"
PROVENANCE_FRESH_VALUE = bytes.fromhex("010200c0ad31699ccf5a92")
PROVENANCE_ANCESTOR_VALUE = bytes.fromhex("010200cf4e9166c4dec912")
BACKUP_EXCLUDE_NAME = b"com.apple.metadata:com_apple_backup_excludeItem"
BACKUP_EXCLUDE_VALUE = bytes.fromhex(
    "62706c69737430305f1011636f6d2e6170706c652e6261636b757064080000000000000101"
    "00000000000000010000000000000000000000000000001c"
)
PROVENANCE_FRESH_ROW = ((PROVENANCE_NAME, PROVENANCE_FRESH_VALUE),)
PROVENANCE_ANCESTOR_ROW = ((PROVENANCE_NAME, PROVENANCE_ANCESTOR_VALUE),)
PROVENANCE_CARGO_ROWS = (
    (BACKUP_EXCLUDE_NAME, BACKUP_EXCLUDE_VALUE),
    (PROVENANCE_NAME, PROVENANCE_FRESH_VALUE),
)

CACHEDIR_TAG_BYTES = (
    b"Signature: 8a477f597d28d172789f06886806bc55\n"
    b"# This file is a cache directory tag created by cargo.\n"
    b"# For information about cache directory tags see https://bford.info/cachedir/\n"
)
CACHEDIR_TAG_SHA256 = "6d9d1d216e0f83abc5e5662ca62c92b4f23009466b54fa27321a69acdb778bb2"

ACL_HOME_PORTABLE = bytes.fromhex(
    "012cc16d000000000000000000000000000000000000000000000000000000000000000000000001"
    "00000000abcdefabcdefabcdefabcdef0000000c0000000200000010"
)
ACL_HOME_NATIVE = bytes.fromhex(
    "6dc12c01000000000000000000000000000000000000000000000000000000000000000001000000"
    "00000000abcdefabcdefabcdefabcdef0000000c0200000010000000"
)
ACL_PRIVATE_TMP_PORTABLE = bytes.fromhex(
    "012cc16d000000000000000000000000000000000000000000000000000000000000000000000001"
    "0000000094868cb291d344208743d81fcebe47a80000006100000a8a"
)
ACL_PRIVATE_TMP_NATIVE = bytes.fromhex(
    "6dc12c01000000000000000000000000000000000000000000000000000000000000000001000000"
    "0000000094868cb291d344208743d81fcebe47a8610000008a0a0000"
)

if (
    len(PROVENANCE_NAME) != 20
    or hashlib.sha256(PROVENANCE_NAME).hexdigest()
    != "2ea77e12ee855235a7baf32be3f31ca8dfc5e3ea11abb9835d23fbfec9dfd420"
    or len(PROVENANCE_FRESH_VALUE) != 11
    or hashlib.sha256(PROVENANCE_FRESH_VALUE).hexdigest()
    != "3ec9f86a978b1be491dd719a8593670b9f341ee45cf8190a5a097052cbd03ca1"
    or len(PROVENANCE_ANCESTOR_VALUE) != 11
    or hashlib.sha256(PROVENANCE_ANCESTOR_VALUE).hexdigest()
    != "790a83a1522d24e7c8ca8b949ee9c36f0ddb9da5c68513b90b8527f40d02a3e5"
    or len(BACKUP_EXCLUDE_NAME) != 47
    or hashlib.sha256(BACKUP_EXCLUDE_NAME).hexdigest()
    != "8270a1c9680263b374e4a2ecbcd0985e796722a2dde54997521bbf405eb39853"
    or len(BACKUP_EXCLUDE_VALUE) != 61
    or hashlib.sha256(BACKUP_EXCLUDE_VALUE).hexdigest()
    != "8332208d45e5ce6a6e8fbce20032850ce228330125d59489170006e91384b7df"
    or len(CACHEDIR_TAG_BYTES) != 177
    or hashlib.sha256(CACHEDIR_TAG_BYTES).hexdigest() != CACHEDIR_TAG_SHA256
    or len(ACL_HOME_PORTABLE) != 68
    or hashlib.sha256(ACL_HOME_PORTABLE).hexdigest()
    != "5fcce06fb1ed90e68bfaf7905c352f7e061b4f2ded7054649ca978f81fe7e0f8"
    or len(ACL_HOME_NATIVE) != 68
    or hashlib.sha256(ACL_HOME_NATIVE).hexdigest()
    != "4330d7ecf77e7ee68d333bd1416b2987e3a27be63723521c9ff58831cbd2c90f"
    or len(ACL_PRIVATE_TMP_PORTABLE) != 68
    or hashlib.sha256(ACL_PRIVATE_TMP_PORTABLE).hexdigest()
    != "6ea8817ad41fa89485410e631bfffc77c7c1146e6b447cec62eb484f3b73ef27"
    or len(ACL_PRIVATE_TMP_NATIVE) != 68
    or hashlib.sha256(ACL_PRIVATE_TMP_NATIVE).hexdigest()
    != "7d6e8a8be3d7c7139a3ee4ee5d008cf775d3b22cb10d3d3284bfc4401a58561a"
):
    raise RuntimeError("frozen Darwin metadata constants are inconsistent")

PAYLOAD_XATTR_SCHEMA = b"schema=c2b2a-darwin-xattr-sidecar-v1\n"
PAYLOAD_XATTR_OBJECTS = (
    ("directory", "."),
    ("directory", ".cargo"),
    ("file", ".cargo/config.toml"),
    ("file", "Cargo.lock"),
    ("file", "Cargo.toml"),
    ("directory", "build-support"),
    ("file", "build-support/controller.py"),
    ("file", "build-support/entropy-failure-probe.cc"),
    ("file", "build-support/entropy-failure-probe.rs"),
    ("file", "build-support/provision.py"),
    ("file", "build-support/root-cargo-config.toml"),
    ("directory", "patches"),
    ("file", "patches/surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch"),
    ("file", "rust-toolchain.toml"),
    ("directory", "src"),
    ("directory", "src/bin"),
    ("file", "src/bin/collector.rs"),
    ("file", "src/bin/supervisor.rs"),
    ("file", "src/contract.rs"),
    ("file", "src/fixtures.rs"),
    ("file", "src/lib.rs"),
    ("file", "src/protocol.rs"),
    ("file", "src/rocks.rs"),
    ("file", "src/seccomp.rs"),
    ("directory", "tests"),
    ("file", "tests/contract.rs"),
    ("file", "tests/protocol.rs"),
    ("file", "tests/rocks.rs"),
    ("file", "tests/seccomp.rs"),
)
PAYLOAD_XATTR_CURRENT_IDENTITY = (
    29,
    59,
    6_937,
    "6a85bb9c4c07f42e8def7868d312769a1cdbbecda4e2d304e51ec47013321742",
)
PAYLOAD_XATTR_MANIFEST_IDENTITY = (
    30,
    61,
    7_193,
    "686b3afc9567237fca0f0c329d5599f69d12773553fce3961afcc0cb1482c18f",
)

APFS_STABLE_PROJECTION = (
    16_777_233,
    (16_777_233, 25),
    0,
    25,
    0x04909080,
    1,
    b"apfs",
    b"/System/Volumes/Data",
    b"/dev/disk3s5",
    1,
)

REPOSITORY_ROOT = "/Users/yuval.meiri/projects/engram"
REPOSITORY_METADATA_PATHS = (
    ("/Users", "users-root-zero", "none"),
    ("/Users/yuval.meiri", "home-zero", "home"),
    ("/Users/yuval.meiri/projects", "projects-ancestor", "none"),
    (REPOSITORY_ROOT, "repository-root-ancestor", "none"),
    (REPOSITORY_ROOT + "/engram-eval", "repository-fresh", "none"),
) + tuple(
    (
        REPOSITORY_PAYLOAD if relative == "." else REPOSITORY_PAYLOAD + "/" + relative,
        "repository-fresh",
        "none",
    )
    for _, relative in PAYLOAD_XATTR_OBJECTS
) + (
    (
        REPOSITORY_PAYLOAD + "/build-support/archive-manifest-v1.tsv",
        "repository-fresh",
        "none",
    ),
)

_BOUND_OWNER: tuple[str, int, int] | None = None
_GENERATED_ROOTS: tuple[str, ...] = ()
_ACQUISITION_REGISTRY: str | None = None
_SYSTEM_METADATA_BINDINGS: dict[str, tuple[object, ...]] = {}
_REPOSITORY_METADATA_BINDINGS: dict[str, tuple[object, ...]] = {}


class _DarwinFsid(ctypes.Structure):
    _fields_ = (("val", ctypes.c_int32 * 2),)


class _DarwinStatFs(ctypes.Structure):
    _fields_ = (
        ("f_bsize", ctypes.c_uint32),
        ("f_iosize", ctypes.c_int32),
        ("f_blocks", ctypes.c_uint64),
        ("f_bfree", ctypes.c_uint64),
        ("f_bavail", ctypes.c_uint64),
        ("f_files", ctypes.c_uint64),
        ("f_ffree", ctypes.c_uint64),
        ("f_fsid", _DarwinFsid),
        ("f_owner", ctypes.c_uint32),
        ("f_type", ctypes.c_uint32),
        ("f_flags", ctypes.c_uint32),
        ("f_fssubtype", ctypes.c_uint32),
        ("f_fstypename", ctypes.c_char * 16),
        ("f_mntonname", ctypes.c_char * 1_024),
        ("f_mntfromname", ctypes.c_char * 1_024),
        ("f_flags_ext", ctypes.c_uint32),
        ("f_reserved", ctypes.c_uint32 * 7),
    )


if (
    ctypes.sizeof(_DarwinStatFs) != STATFS_BYTES
    or ctypes.alignment(_DarwinStatFs) != 8
    or tuple(getattr(_DarwinStatFs, name).offset for name in (
        "f_bsize",
        "f_iosize",
        "f_blocks",
        "f_bfree",
        "f_bavail",
        "f_files",
        "f_ffree",
        "f_fsid",
        "f_owner",
        "f_type",
        "f_flags",
        "f_fssubtype",
        "f_fstypename",
        "f_mntonname",
        "f_mntfromname",
        "f_flags_ext",
        "f_reserved",
    ))
    != (0, 4, 8, 16, 24, 32, 40, 48, 56, 60, 64, 68, 72, 88, 1_112, 2_136, 2_140)
):
    raise RuntimeError("reviewed Darwin statfs ABI layout is unavailable")

_LIBC = ctypes.CDLL(None, use_errno=True)
try:
    _LIBC.flistxattr.argtypes = (
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_size_t,
        ctypes.c_int,
    )
    _LIBC.flistxattr.restype = ctypes.c_ssize_t
    _LIBC.fgetxattr.argtypes = (
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_void_p,
        ctypes.c_size_t,
        ctypes.c_uint32,
        ctypes.c_int,
    )
    _LIBC.fgetxattr.restype = ctypes.c_ssize_t
    _LIBC.fstatfs.argtypes = (ctypes.c_int, ctypes.POINTER(_DarwinStatFs))
    _LIBC.fstatfs.restype = ctypes.c_int
    _LIBC.acl_get_fd_np.argtypes = (ctypes.c_int, ctypes.c_int)
    _LIBC.acl_get_fd_np.restype = ctypes.c_void_p
    _LIBC.acl_size.argtypes = (ctypes.c_void_p,)
    _LIBC.acl_size.restype = ctypes.c_ssize_t
    _LIBC.acl_copy_ext.argtypes = (ctypes.c_void_p, ctypes.c_void_p, ctypes.c_ssize_t)
    _LIBC.acl_copy_ext.restype = ctypes.c_ssize_t
    _LIBC.acl_copy_ext_native.argtypes = (
        ctypes.c_void_p,
        ctypes.c_void_p,
        ctypes.c_ssize_t,
    )
    _LIBC.acl_copy_ext_native.restype = ctypes.c_ssize_t
    _LIBC.acl_get_entry.argtypes = (
        ctypes.c_void_p,
        ctypes.c_int,
        ctypes.POINTER(ctypes.c_void_p),
    )
    _LIBC.acl_get_entry.restype = ctypes.c_int
    _LIBC.acl_free.argtypes = (ctypes.c_void_p,)
    _LIBC.acl_free.restype = ctypes.c_int
except AttributeError as exc:
    raise RuntimeError("required descriptor ACL interface is unavailable") from exc


class ProvisionError(RuntimeError):
    """Terminal failure in one closed provisioning mode."""


def provisioner_expected_xattrs(profile):
    if profile == "zero":
        return ()
    if profile == "provenance-fresh":
        return ((b"com.apple.provenance", b"\x01\x02\x00\xc0\xad\x31\x69\x9c\xcf\x5a\x92"),)
    if profile == "provenance-ancestor":
        return ((b"com.apple.provenance", b"\x01\x02\x00\xcf\x4e\x91\x66\xc4\xde\xc9\x12"),)
    if profile == "provenance-plus-backup":
        return (
            (
                b"com.apple.metadata:com_apple_backup_excludeItem",
                b"bplist00_\x10\x11com.apple.backupd\x08\x00\x00\x00\x00\x00\x00\x01\x01"
                b"\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x00\x00\x00\x00\x00"
                b"\x00\x00\x00\x00\x00\x00\x00\x1c",
            ),
            (b"com.apple.provenance", b"\x01\x02\x00\xc0\xad\x31\x69\x9c\xcf\x5a\x92"),
        )
    raise ValueError("unknown provisioner xattr profile")


def provisioner_profile_accepts(attributes, profile):
    try:
        expected = provisioner_expected_xattrs(profile)
    except ValueError:
        return False
    return tuple(attributes) == expected


def provisioner_object_profile(object_class, fd):
    if not isinstance(fd, int) or isinstance(fd, bool) or fd < 0:
        raise ValueError("invalid provisioner metadata descriptor")
    if object_class == "system-parent":
        return "zero", True, int(fd)
    if object_class in ("users-root-zero", "home-zero"):
        return "zero", False, int(fd)
    if object_class in ("repository-fresh", "generated", "cargo-ordinary"):
        return "provenance-fresh", False, int(fd)
    if object_class in ("projects-ancestor", "repository-root-ancestor"):
        return "provenance-ancestor", False, int(fd)
    if object_class == "acquisition-registry":
        return "provenance-plus-backup", False, int(fd)
    raise ValueError("unknown provisioner metadata object class")


def provisioner_payload_sidecar_identity(manifest_present):
    if manifest_present is False:
        return (
            False,
            (
                29,
                59,
                6937,
                "6a85bb9c4c07f42e8def7868d312769a1cdbbecda4e2d304e51ec47013321742",
            ),
        )
    if manifest_present is True:
        return (
            True,
            (
                30,
                61,
                7193,
                "686b3afc9567237fca0f0c329d5599f69d12773553fce3961afcc0cb1482c18f",
            ),
        )
    raise ValueError("payload manifest-presence selector is not boolean")


def provisioner_accept_payload_sidecar(stream, objects, rows, identity):
    if (
        not isinstance(stream, bytes)
        or not isinstance(objects, int)
        or isinstance(objects, bool)
        or not isinstance(rows, int)
        or isinstance(rows, bool)
        or not isinstance(identity, tuple)
        or len(identity) != 4
    ):
        raise ValueError("payload sidecar acceptance shape is invalid")
    expected_objects, expected_rows, expected_bytes, expected_sha256 = identity
    if (
        objects != expected_objects
        or rows != expected_rows
        or stream.count(b"\n") != expected_rows
        or len(stream) != expected_bytes
        or hashlib.sha256(stream).hexdigest() != expected_sha256
    ):
        raise ValueError("payload sidecar differs from its canonical identity")
    return bytes(stream)


def provisioner_cargo_marker_accepts(object_class, attributes, tag_size, tag_sha256):
    profile_accepted = provisioner_profile_accepts(
        attributes,
        "provenance-plus-backup",
    )
    return (
        object_class == "acquisition-registry"
        and profile_accepted
        and tag_size == 177
        and tag_sha256
        == "6d9d1d216e0f83abc5e5662ca62c92b4f23009466b54fa27321a69acdb778bb2"
    )


def fail(message: str) -> None:
    raise ProvisionError(message)


def checked_add(left: int, right: int, limit: int) -> int:
    if left < 0 or right < 0 or left > limit - right:
        fail("bounded arithmetic overflow")
    return left + right


def raw_key(value: str) -> bytes:
    try:
        encoded = value.encode("utf-8", "strict")
    except UnicodeError as exc:
        raise ProvisionError("path or field is not strict UTF-8") from exc
    if len(encoded) > PATH_BYTES_MAX:
        fail("UTF-8 path exceeds frozen bound")
    return encoded


def reject_controls(value: str, *, allow_slash: bool) -> None:
    raw = raw_key(value)
    if not raw or b"\\" in raw:
        fail("empty value or backslash is forbidden")
    if not allow_slash and b"/" in raw:
        fail("path separator is forbidden in a component")
    if any(byte < 0x20 or byte == 0x7F for byte in raw):
        fail("control byte is forbidden")


def validate_relative(path: str) -> tuple[str, ...]:
    reject_controls(path, allow_slash=True)
    if path.startswith("/") or path.endswith("/"):
        fail("relative path has an absolute or empty component")
    parts = tuple(path.split("/"))
    if len(parts) > PATH_COMPONENTS_MAX or any(part in ("", ".", "..") for part in parts):
        fail("relative path components are invalid")
    for part in parts:
        reject_controls(part, allow_slash=False)
    return parts


def validate_absolute(path: str, *, leaf_may_be_absent: bool = False) -> str:
    raw_key(path)
    if not path.startswith("/") or path == "/" or path.endswith("/") or "//" in path:
        fail("path is not canonical absolute")
    parts = tuple(path.split("/")[1:])
    if len(parts) > PATH_COMPONENTS_MAX or any(part in ("", ".", "..") for part in parts):
        fail("absolute path components are invalid")
    for part in parts:
        reject_controls(part, allow_slash=False)
    stop = len(parts) - 1
    fd = os.open("/", os.O_RDONLY | O_DIRECTORY | O_CLOEXEC)
    try:
        root_info = require_traversable_directory_fd(fd)
        require_absolute_metadata("/", fd, root_info, None)
        traversed = ""
        carried_object_class: str | None = None
        for part in parts[:stop]:
            child_path = traversed + "/" + part
            rule = metadata_rule_for_absolute(child_path, carried_object_class)
            child = os.open(part, os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC, dir_fd=fd)
            os.close(fd)
            fd = child
            info = require_traversable_directory_fd(fd)
            traversed = child_path
            require_absolute_metadata(traversed, fd, info, rule)
            require_bound_owner_identity(traversed, info)
            if rule is not None and rule[0] in ("generated", "acquisition-registry"):
                carried_object_class = "generated"
        if not leaf_may_be_absent:
            info = os.stat(parts[-1], dir_fd=fd, follow_symlinks=False)
            if stat.S_ISLNK(info.st_mode):
                fail("path leaf is a symlink")
    except OSError as exc:
        raise ProvisionError("absolute path validation failed") from exc
    finally:
        os.close(fd)
    return path


def split_new_path(path: str) -> tuple[int, str]:
    validate_absolute(path, leaf_may_be_absent=True)
    parent, leaf = os.path.split(path)
    reject_controls(leaf, allow_slash=False)
    parent_fd = open_directory_absolute(parent)
    try:
        try:
            os.stat(leaf, dir_fd=parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            return os.dup(parent_fd), leaf
        fail("create-new output already exists")
    finally:
        os.close(parent_fd)


def open_directory_absolute(path: str) -> int:
    validate_absolute(path)
    fd = os.open("/", os.O_RDONLY | O_DIRECTORY | O_CLOEXEC)
    try:
        root_info = require_traversable_directory_fd(fd)
        require_absolute_metadata("/", fd, root_info, None)
        traversed = ""
        carried_object_class: str | None = None
        for part in path.split("/")[1:]:
            child_path = traversed + "/" + part
            rule = metadata_rule_for_absolute(child_path, carried_object_class)
            child = os.open(part, os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC, dir_fd=fd)
            os.close(fd)
            fd = child
            info = require_traversable_directory_fd(fd)
            traversed = child_path
            require_absolute_metadata(traversed, fd, info, rule)
            require_bound_owner_identity(traversed, info)
            if rule is not None and rule[0] in ("generated", "acquisition-registry"):
                carried_object_class = "generated"
        return fd
    except BaseException:
        os.close(fd)
        raise


def open_regular_absolute(
    path: str,
    *,
    object_class: str,
    maximum: int = U64_MAX,
) -> tuple[int, os.stat_result]:
    validate_absolute(path)
    parent, leaf = os.path.split(path)
    parent_fd = open_directory_absolute(parent)
    try:
        fd = os.open(leaf, os.O_RDONLY | O_NOFOLLOW | O_CLOEXEC, dir_fd=parent_fd)
    except OSError as exc:
        raise ProvisionError("regular input open failed") from exc
    finally:
        os.close(parent_fd)
    try:
        info = require_regular_fd(fd, maximum=maximum, object_class=object_class)
        return fd, info
    except BaseException:
        os.close(fd)
        raise


def require_owner(info: os.stat_result) -> None:
    if info.st_uid != os.getuid() or info.st_gid != os.getgid():
        fail("filesystem object is not owned by the invoking identity")


def require_owner_only_mode(info: os.stat_result) -> None:
    if stat.S_IMODE(info.st_mode) & 0o077:
        fail("Cargo-generated cache object is not owner-only")


def require_traversable_directory_fd(fd: int) -> os.stat_result:
    info = os.fstat(fd)
    if not stat.S_ISDIR(info.st_mode):
        fail("absolute-path ancestor is not a directory")
    return info


def require_bound_owner_identity(path: str, info: os.stat_result) -> None:
    if _BOUND_OWNER is None or path != _BOUND_OWNER[0]:
        return
    if (info.st_dev, info.st_ino) != (_BOUND_OWNER[1], _BOUND_OWNER[2]):
        fail("fresh owner-root path no longer names its pre-opened inode")


def bind_owner(path: str, info: os.stat_result) -> None:
    global _BOUND_OWNER
    if _BOUND_OWNER is not None:
        fail("fresh owner root was bound more than once")
    _BOUND_OWNER = (path, info.st_dev, info.st_ino)


def begin_metadata_scope(mode: str, owner: str) -> None:
    global _GENERATED_ROOTS
    if _GENERATED_ROOTS:
        fail("provisioner metadata scope was bound more than once")
    if mode in ("lock-payload", "lock-source"):
        _GENERATED_ROOTS = (os.path.dirname(owner),)
    else:
        _GENERATED_ROOTS = (owner,)


def add_generated_root(path: str) -> None:
    global _GENERATED_ROOTS
    if path in _GENERATED_ROOTS:
        return
    if any(is_beneath(path, root) or is_beneath(root, path) for root in _GENERATED_ROOTS):
        fail("generated metadata roots unexpectedly overlap")
    _GENERATED_ROOTS += (path,)


def bind_acquisition_registry(path: str) -> None:
    global _ACQUISITION_REGISTRY
    if _ACQUISITION_REGISTRY is not None:
        fail("acquisition registry metadata class was bound more than once")
    _ACQUISITION_REGISTRY = path


def stat_identity(info: os.stat_result) -> tuple[object, ...]:
    flags = getattr(info, "st_flags", None)
    birthtime = getattr(info, "st_birthtime", None)
    if flags is None or birthtime is None:
        fail("reviewed Darwin stat fields are unavailable")
    return (
        info.st_dev,
        info.st_ino,
        info.st_mode,
        info.st_nlink,
        info.st_uid,
        info.st_gid,
        info.st_size,
        info.st_mtime_ns,
        info.st_ctime_ns,
        flags,
        birthtime,
    )


def parent_stable_identity(info: os.stat_result) -> tuple[object, ...]:
    flags = getattr(info, "st_flags", None)
    if flags is None:
        fail("reviewed Darwin parent stat fields are unavailable")
    return (
        info.st_dev,
        info.st_ino,
        stat.S_IFMT(info.st_mode),
        stat.S_IMODE(info.st_mode),
        info.st_uid,
        info.st_gid,
        flags,
    )


def fstatfs_projection(fd: int) -> tuple[object, ...]:
    filesystem = _DarwinStatFs()
    ctypes.set_errno(0)
    if _LIBC.fstatfs(fd, ctypes.byref(filesystem)) != 0:
        fail("descriptor filesystem inspection failed")

    def bounded_c_string(offset: int, size: int) -> bytes:
        raw = ctypes.string_at(ctypes.addressof(filesystem) + offset, size)
        if b"\x00" not in raw:
            fail("descriptor filesystem string is not NUL terminated")
        return raw.split(b"\x00", 1)[0]

    return (
        os.fstat(fd).st_dev,
        (int(filesystem.f_fsid.val[0]), int(filesystem.f_fsid.val[1])),
        int(filesystem.f_owner),
        int(filesystem.f_type),
        int(filesystem.f_flags),
        int(filesystem.f_fssubtype),
        bounded_c_string(_DarwinStatFs.f_fstypename.offset, 16),
        bounded_c_string(_DarwinStatFs.f_mntonname.offset, 1_024),
        bounded_c_string(_DarwinStatFs.f_mntfromname.offset, 1_024),
        int(filesystem.f_flags_ext),
    )


def require_apfs_fd(fd: int) -> tuple[object, ...]:
    projection = fstatfs_projection(fd)
    if projection != APFS_STABLE_PROJECTION:
        fail("descriptor is not on the exact reviewed writable APFS data volume")
    if projection[4] & MNT_IGNORE_OWNERSHIP:
        fail("APFS ownership enforcement is disabled")
    return projection


def xattr_names_fd(
    fd: int,
    *,
    name_list_maximum: int,
    count_maximum: int,
) -> tuple[bytes, ...]:
    size = _LIBC.flistxattr(fd, None, 0, 0)
    if size < 0:
        fail("extended-attribute name-list sizing failed")
    if size == 0:
        return ()
    if size > name_list_maximum:
        fail("extended-attribute name list exceeds its frozen bound")
    buffer = ctypes.create_string_buffer(size)
    observed = _LIBC.flistxattr(fd, buffer, size, 0)
    if observed != size:
        fail("extended-attribute name list changed while reading")
    raw = bytes(buffer.raw[:size])
    if not raw.endswith(b"\x00"):
        fail("extended-attribute name list is malformed")
    names = tuple(raw.split(b"\x00")[:-1])
    if (
        not all(name and len(name) <= 255 and b"\x00" not in name for name in names)
        or len(names) != len(set(names))
        or len(names) > count_maximum
    ):
        fail("extended-attribute name table is invalid")
    return tuple(sorted(names))


def xattr_value_fd(fd: int, name: bytes, *, maximum: int) -> bytes:
    size = _LIBC.fgetxattr(fd, name, None, 0, 0, 0)
    if size < 0 or size > maximum:
        fail("extended-attribute value sizing failed or exceeded its bound")
    if size == 0:
        return b""
    buffer = ctypes.create_string_buffer(size)
    observed = _LIBC.fgetxattr(fd, name, buffer, size, 0, 0)
    if observed != size:
        fail("extended-attribute value changed while reading")
    return bytes(buffer.raw[:size])


def xattr_snapshot_once_fd(
    fd: int,
    *,
    system_input: bool,
) -> tuple[tuple[bytes, bytes], ...]:
    name_list_maximum = (
        SYSTEM_XATTR_NAME_LIST_MAX if system_input else XATTR_NAME_LIST_MAX
    )
    count_maximum = SYSTEM_XATTR_COUNT_MAX if system_input else 2
    value_maximum = SYSTEM_XATTR_VALUE_MAX if system_input else XATTR_VALUE_MAX
    total_maximum = (
        SYSTEM_XATTR_VALUES_TOTAL_MAX
        if system_input
        else XATTR_VALUE_MAX + len(PROVENANCE_FRESH_VALUE)
    )
    names = xattr_names_fd(
        fd,
        name_list_maximum=name_list_maximum,
        count_maximum=count_maximum,
    )
    attributes: list[tuple[bytes, bytes]] = []
    total = 0
    for name in names:
        value = xattr_value_fd(fd, name, maximum=value_maximum)
        total = checked_add(total, len(value), total_maximum)
        attributes.append((name, value))
    if xattr_names_fd(
        fd,
        name_list_maximum=name_list_maximum,
        count_maximum=count_maximum,
    ) != names:
        fail("extended-attribute name table changed while reading")
    return tuple(attributes)


def xattrs_fd(
    fd: int,
    *,
    system_input: bool,
    private_tmp_parent: bool = False,
) -> tuple[tuple[bytes, bytes], ...]:
    identity_projection = parent_stable_identity if private_tmp_parent else stat_identity
    before = os.fstat(fd)
    first = xattr_snapshot_once_fd(fd, system_input=system_input)
    second = xattr_snapshot_once_fd(fd, system_input=system_input)
    after = os.fstat(fd)
    if identity_projection(before) != identity_projection(after) or first != second:
        fail("extended-attribute snapshot or object identity changed while reading")
    return first


def acl_snapshot_once_fd(fd: int) -> tuple[bool, bytes, bytes, int]:
    ctypes.set_errno(0)
    acl = _LIBC.acl_get_fd_np(fd, ACL_TYPE_EXTENDED)
    if not acl:
        number = ctypes.get_errno()
        if number == errno.ENOENT:
            return False, b"", b"", 0
        fail("extended ACL cannot be inspected")
    try:
        size = _LIBC.acl_size(acl)
        if size <= 0 or size > ACL_EXTERNAL_MAX:
            fail("extended ACL external size is invalid")
        portable_buffer = ctypes.create_string_buffer(size)
        portable_size = _LIBC.acl_copy_ext(portable_buffer, acl, size)
        if portable_size != size:
            fail("portable extended ACL serialization failed")
        native_buffer = ctypes.create_string_buffer(size)
        native_size = _LIBC.acl_copy_ext_native(native_buffer, acl, size)
        if native_size != size:
            fail("native extended ACL serialization failed")
        entry = ctypes.c_void_p()
        ctypes.set_errno(0)
        result = _LIBC.acl_get_entry(acl, ACL_FIRST_ENTRY, ctypes.byref(entry))
        if result != 0:
            fail("extended ACL cannot be enumerated")
        count = 1
        while True:
            ctypes.set_errno(0)
            result = _LIBC.acl_get_entry(acl, ACL_NEXT_ENTRY, ctypes.byref(entry))
            if result == -1 and ctypes.get_errno() == errno.EINVAL:
                break
            if result != 0:
                fail("extended ACL cannot be enumerated")
            count = checked_add(count, 1, MANIFEST_ROWS_MAX)
        return (
            True,
            bytes(portable_buffer.raw[:portable_size]),
            bytes(native_buffer.raw[:native_size]),
            count,
        )
    finally:
        if _LIBC.acl_free(acl) != 0:
            fail("extended ACL cannot be released")


def acl_snapshot_fd(
    fd: int,
    *,
    private_tmp_parent: bool = False,
) -> tuple[bool, bytes, bytes, int]:
    identity_projection = parent_stable_identity if private_tmp_parent else stat_identity
    before = os.fstat(fd)
    first = acl_snapshot_once_fd(fd)
    second = acl_snapshot_once_fd(fd)
    after = os.fstat(fd)
    if identity_projection(before) != identity_projection(after) or first != second:
        fail("extended ACL snapshot or object identity changed while reading")
    return first


def require_acl_profile(fd: int, profile: str) -> tuple[bool, bytes, bytes, int]:
    observed = acl_snapshot_fd(fd, private_tmp_parent=profile == "private-tmp")
    if profile == "none":
        expected = (False, b"", b"", 0)
    elif profile == "home":
        expected = (True, ACL_HOME_PORTABLE, ACL_HOME_NATIVE, 1)
    elif profile == "private-tmp":
        expected = (True, ACL_PRIVATE_TMP_PORTABLE, ACL_PRIVATE_TMP_NATIVE, 1)
    else:
        fail("unknown extended ACL profile")
    if observed != expected:
        fail("extended ACL differs from its exact closed profile")
    return expected


def require_xattr_class(fd: int, object_class: str) -> None:
    try:
        profile, private_tmp_parent, accepted_fd = provisioner_object_profile(object_class, fd)
    except ValueError as exc:
        raise ProvisionError("unknown provisioner metadata object class") from exc
    object_class = None
    fd = None
    attributes = xattrs_fd(
        accepted_fd,
        system_input=False,
        private_tmp_parent=private_tmp_parent,
    )
    profile_accepted = provisioner_profile_accepts(attributes, profile)
    if not profile_accepted:
        fail("extended attributes differ from their exact closed profile")


def require_plain_metadata(
    fd: int,
    info: os.stat_result,
    *,
    object_class: str,
    acl_profile: str = "none",
    owner_required: bool = True,
) -> None:
    if owner_required:
        require_owner(info)
    require_apfs_fd(fd)
    require_xattr_class(fd, object_class)
    require_acl_profile(fd, acl_profile)
    if info.st_mode & (stat.S_ISUID | stat.S_ISGID | stat.S_ISVTX):
        fail("set-id or sticky mode bits are forbidden")
    if stat_identity(info) != stat_identity(os.fstat(fd)):
        fail("filesystem object changed during metadata validation")


def require_private_tmp_parent_fd(fd: int) -> tuple[os.stat_result, tuple[object, ...]]:
    info = os.fstat(fd)
    if (
        not stat.S_ISDIR(info.st_mode)
        or info.st_uid != 0
        or info.st_gid != 0
        or stat.S_IMODE(info.st_mode) != 0o1777
    ):
        fail("/private/tmp metadata differs from the reviewed system-parent class")
    require_apfs_fd(fd)
    require_xattr_class(fd, "system-parent")
    require_acl_profile(fd, "private-tmp")
    if parent_stable_identity(info) != parent_stable_identity(os.fstat(fd)):
        fail("/private/tmp stable metadata changed during validation")
    return info, parent_stable_identity(info)


def metadata_rule_for_absolute(
    path: str,
    carried_object_class: str | None,
) -> tuple[str, str] | None:
    for expected_path, object_class, acl_profile in REPOSITORY_METADATA_PATHS:
        if path == expected_path:
            return object_class, acl_profile
    if path == "/private/tmp":
        return "system-parent", "private-tmp"
    if path in ("/private", "/private/var", "/private/var/empty"):
        return None
    if _ACQUISITION_REGISTRY is not None and path == _ACQUISITION_REGISTRY:
        return "acquisition-registry", "none"
    if path in _GENERATED_ROOTS:
        return "generated", "none"
    if carried_object_class == "generated":
        return "generated", "none"
    if carried_object_class is not None:
        fail("unknown carried metadata object class")
    fail("absolute path is outside the reviewed provisioner metadata closure")


def system_metadata_snapshot(fd: int, info: os.stat_result) -> tuple[object, ...]:
    if info.st_uid != 0 or stat.S_IMODE(info.st_mode) & 0o022:
        fail("system-input traversal object is not root-owned and non-writable")
    identity = stat_identity(info)
    observed = (
        identity,
        fstatfs_projection(fd),
        xattrs_fd(fd, system_input=True),
        acl_snapshot_fd(fd),
    )
    if identity != stat_identity(os.fstat(fd)):
        fail("system-input metadata changed while binding its complete snapshot")
    return observed


def repository_metadata_snapshot(fd: int, info: os.stat_result) -> tuple[object, ...]:
    require_owner(info)
    identity = stat_identity(info)
    observed = (
        identity,
        fstatfs_projection(fd),
        xattrs_fd(fd, system_input=False),
        acl_snapshot_fd(fd),
    )
    if identity != stat_identity(os.fstat(fd)):
        fail("repository metadata changed while binding its complete snapshot")
    return observed


def require_absolute_metadata(
    path: str,
    fd: int,
    info: os.stat_result,
    rule: tuple[str, str] | None,
) -> None:
    if rule is None:
        observed = system_metadata_snapshot(fd, info)
        previous = _SYSTEM_METADATA_BINDINGS.setdefault(path, observed)
        if previous != observed:
            fail("preflight-bound system-input metadata changed during provisioning")
        return
    object_class, acl_profile = rule
    if object_class == "system-parent":
        require_private_tmp_parent_fd(fd)
        return
    require_plain_metadata(
        fd,
        info,
        object_class=object_class,
        acl_profile=acl_profile,
        owner_required=object_class != "users-root-zero",
    )
    if object_class == "users-root-zero":
        observed = system_metadata_snapshot(fd, os.fstat(fd))
        previous = _REPOSITORY_METADATA_BINDINGS.setdefault(path, observed)
        if previous != observed:
            fail("repository metadata changed during provisioning")
    elif object_class in (
        "home-zero",
        "projects-ancestor",
        "repository-root-ancestor",
        "repository-fresh",
    ):
        observed = repository_metadata_snapshot(fd, os.fstat(fd))
        previous = _REPOSITORY_METADATA_BINDINGS.setdefault(path, observed)
        if previous != observed:
            fail("repository metadata changed during provisioning")


def require_directory_fd(
    fd: int,
    *,
    object_class: str,
    mode: int | None = None,
    empty: bool = False,
) -> os.stat_result:
    info = os.fstat(fd)
    if not stat.S_ISDIR(info.st_mode):
        fail("object is not a directory")
    require_plain_metadata(fd, info, object_class=object_class)
    if mode is not None and stat.S_IMODE(info.st_mode) != mode:
        fail("directory mode is not exact")
    if empty and os.listdir(fd):
        fail("directory is not empty")
    return info


def require_regular_fd(
    fd: int,
    *,
    object_class: str,
    maximum: int = U64_MAX,
) -> os.stat_result:
    info = os.fstat(fd)
    if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
        fail("object is not a single-link regular file")
    require_plain_metadata(fd, info, object_class=object_class)
    if info.st_size < 0 or info.st_size > maximum:
        fail("regular file exceeds its frozen bound")
    if info.st_blocks * 512 < info.st_size:
        fail("sparse regular files are forbidden")
    return info


def stable_identity(left: os.stat_result, right: os.stat_result) -> bool:
    return stat_identity(left) == stat_identity(right)


def hash_open_file(
    fd: int,
    expected: os.stat_result | None = None,
    *,
    object_class: str,
) -> tuple[int, str]:
    before = require_regular_fd(fd, object_class=object_class)
    if expected is not None and not stable_identity(before, expected):
        fail("regular file identity changed before hashing")
    os.lseek(fd, 0, os.SEEK_SET)
    digest = hashlib.sha256()
    size = 0
    while True:
        block = os.read(fd, READ_CHUNK)
        if not block:
            break
        size = checked_add(size, len(block), U64_MAX)
        digest.update(block)
    after = require_regular_fd(fd, object_class=object_class)
    if not stable_identity(before, after) or size != before.st_size:
        fail("regular file changed while hashing")
    return size, digest.hexdigest()


def read_open_file(
    fd: int,
    maximum: int,
    *,
    object_class: str,
) -> tuple[bytes, os.stat_result]:
    before = require_regular_fd(fd, maximum=maximum, object_class=object_class)
    os.lseek(fd, 0, os.SEEK_SET)
    chunks: list[bytes] = []
    size = 0
    while True:
        block = os.read(fd, min(READ_CHUNK, maximum + 1 - size))
        if not block:
            break
        size = checked_add(size, len(block), maximum)
        chunks.append(block)
    after = require_regular_fd(fd, maximum=maximum, object_class=object_class)
    if not stable_identity(before, after) or size != before.st_size:
        fail("regular file changed while reading")
    return b"".join(chunks), before


def read_regular_absolute(
    path: str,
    maximum: int,
    *,
    object_class: str,
) -> bytes:
    fd, _ = open_regular_absolute(
        path,
        maximum=maximum,
        object_class=object_class,
    )
    try:
        data, _ = read_open_file(fd, maximum, object_class=object_class)
        return data
    finally:
        os.close(fd)


@dataclass(frozen=True)
class LockPackage:
    name: str
    version: str
    source: str
    checksum: str

    @property
    def basename(self) -> str:
        value = self.name + "-" + self.version + ".crate"
        reject_controls(value, allow_slash=False)
        return value


@dataclass(frozen=True)
class ArchiveRow:
    name: str
    version: str
    source: str
    basename: str
    size: int
    checksum: str

    @property
    def crate_root(self) -> str:
        value = self.name + "-" + self.version
        reject_controls(value, allow_slash=False)
        return value


def decode_utf8(data: bytes, description: str) -> str:
    if data.startswith(b"\xef\xbb\xbf"):
        fail(description + " has a BOM")
    try:
        return data.decode("utf-8", "strict")
    except UnicodeError as exc:
        raise ProvisionError(description + " is not strict UTF-8") from exc


def require_sha256(value: str) -> str:
    if re.fullmatch(r"[0-9a-f]{64}", value) is None:
        fail("SHA-256 field is not canonical lowercase hex")
    return value


def require_decimal(value: str, maximum: int) -> int:
    if re.fullmatch(r"0|[1-9][0-9]*", value) is None:
        fail("decimal field is not canonical")
    number = int(value, 10)
    if number > maximum:
        fail("decimal field exceeds its bound")
    return number


def parse_lock_bytes(data: bytes) -> tuple[LockPackage, ...]:
    text = decode_utf8(data, "Cargo.lock")
    try:
        parsed = tomllib.loads(text)
    except tomllib.TOMLDecodeError as exc:
        raise ProvisionError("Cargo.lock is not valid TOML") from exc
    if parsed.get("version") != 4 or not isinstance(parsed.get("package"), list):
        fail("Cargo.lock schema is not the pinned v4 package schema")
    registry: list[LockPackage] = []
    roots: list[tuple[str, str]] = []
    seen: set[tuple[str, str, str]] = set()
    basenames: set[str] = set()
    for item in parsed["package"]:
        if not isinstance(item, dict):
            fail("Cargo.lock package row is not a table")
        name = item.get("name")
        version = item.get("version")
        source = item.get("source")
        checksum = item.get("checksum")
        if not isinstance(name, str) or not isinstance(version, str):
            fail("Cargo.lock package identity is malformed")
        reject_controls(name, allow_slash=False)
        reject_controls(version, allow_slash=False)
        if source is None:
            if checksum is not None:
                fail("path package unexpectedly has a checksum")
            roots.append((name, version))
            continue
        if source != REGISTRY_SOURCE or not isinstance(checksum, str):
            fail("Cargo.lock contains a non-authorized package source")
        require_sha256(checksum)
        package = LockPackage(name, version, source, checksum)
        key = (name, version, source)
        if key in seen or package.basename in basenames:
            fail("Cargo.lock contains a duplicate package or archive basename")
        seen.add(key)
        basenames.add(package.basename)
        registry.append(package)
    if roots != [ROOT_PACKAGE]:
        fail("Cargo.lock does not contain exactly the sole pinned path root")
    if not registry or len(registry) > MANIFEST_ROWS_MAX:
        fail("Cargo.lock registry package count is outside the frozen bound")
    registry.sort(key=lambda row: (raw_key(row.name), raw_key(row.version), raw_key(row.source)))
    return tuple(registry)


def encode_manifest(rows: Sequence[ArchiveRow]) -> bytes:
    if not rows or len(rows) > MANIFEST_ROWS_MAX:
        fail("manifest package row count is outside the frozen bound")
    ordered = sorted(
        rows,
        key=lambda row: (raw_key(row.name), raw_key(row.version), raw_key(row.source)),
    )
    if list(rows) != ordered:
        fail("manifest rows are not in canonical order")
    result = bytearray(MANIFEST_SCHEMA)
    seen: set[tuple[str, str, str]] = set()
    basenames: set[str] = set()
    for row in rows:
        fields = (row.name, row.version, row.source, row.basename, str(row.size), row.checksum)
        for index, field in enumerate(fields):
            reject_controls(field, allow_slash=index == 2)
        require_sha256(row.checksum)
        require_decimal(str(row.size), ARCHIVE_BYTES_MAX)
        if row.source != REGISTRY_SOURCE or row.basename != row.name + "-" + row.version + ".crate":
            fail("manifest package row does not match its registry identity")
        key = (row.name, row.version, row.source)
        if key in seen or row.basename in basenames:
            fail("manifest contains a duplicate package or archive basename")
        seen.add(key)
        basenames.add(row.basename)
        line = "package\t" + "\t".join(fields) + "\n"
        result.extend(line.encode("utf-8", "strict"))
        if len(result) > MANIFEST_BYTES_MAX:
            fail("manifest exceeds its byte bound")
    return bytes(result)


def parse_manifest_bytes(data: bytes) -> tuple[ArchiveRow, ...]:
    if not data or len(data) > MANIFEST_BYTES_MAX or not data.endswith(b"\n"):
        fail("manifest byte shape is invalid")
    text = decode_utf8(data, "archive manifest")
    lines = text.splitlines(keepends=True)
    if not lines or lines[0].encode("utf-8") != MANIFEST_SCHEMA:
        fail("manifest schema row is not exact")
    if len(lines) - 1 < 1 or len(lines) - 1 > MANIFEST_ROWS_MAX:
        fail("manifest package row count is outside the frozen bound")
    rows: list[ArchiveRow] = []
    for line in lines[1:]:
        if not line.endswith("\n") or line.endswith("\r\n"):
            fail("manifest row terminator is not canonical LF")
        fields = line[:-1].split("\t")
        if len(fields) != 7 or fields[0] != "package" or any(not field for field in fields):
            fail("manifest package row shape is invalid")
        _, name, version, source, basename, size_text, checksum = fields
        for index, field in enumerate(fields):
            reject_controls(field, allow_slash=index == 3)
        row = ArchiveRow(
            name,
            version,
            source,
            basename,
            require_decimal(size_text, ARCHIVE_BYTES_MAX),
            require_sha256(checksum),
        )
        rows.append(row)
    encoded = encode_manifest(rows)
    if encoded != data:
        fail("manifest bytes are not canonical")
    return tuple(rows)


def read_manifest(path: str) -> tuple[bytes, tuple[ArchiveRow, ...]]:
    parent_fd = open_directory_absolute(os.path.dirname(path))
    try:
        parent = require_directory_fd(
            parent_fd,
            mode=0o500,
            object_class="generated",
        )
        if parent.st_mtime_ns != 0:
            fail("manifest authority directory mtime is not epoch zero")
    finally:
        os.close(parent_fd)
    fd, before = open_regular_absolute(
        path,
        maximum=MANIFEST_BYTES_MAX,
        object_class="generated",
    )
    try:
        if stat.S_IMODE(before.st_mode) != 0o400 or before.st_mtime_ns != 0:
            fail("manifest authority is not sealed mode 0400 at epoch zero")
        data, after = read_open_file(
            fd,
            MANIFEST_BYTES_MAX,
            object_class="generated",
        )
        if not stable_identity(before, after):
            fail("manifest authority changed while reading")
        return data, parse_manifest_bytes(data)
    finally:
        os.close(fd)


def compare_manifest_to_lock(rows: Sequence[ArchiveRow], packages: Sequence[LockPackage]) -> None:
    manifest_identity = [(row.name, row.version, row.source, row.checksum) for row in rows]
    lock_identity = [(row.name, row.version, row.source, row.checksum) for row in packages]
    if manifest_identity != lock_identity:
        fail("archive manifest does not exactly cover Cargo.lock")


def phase_owner(mode: str, values: Mapping[str, str]) -> tuple[str, str]:
    if mode == "acquisition-stage":
        owner = os.path.dirname(values["archive_staging"])
        if os.path.dirname(values["manifest_authority"]) != owner:
            fail("acquisition durable outputs do not share their owner root")
        phase = "acquisition-stage"
    elif mode == "archive-seal":
        owner = os.path.dirname(values["destination"])
        phase = "archive-seal"
    elif mode in ("lock-payload", "lock-source"):
        owner = (
            values["destination_root"]
            if mode == "lock-payload"
            else os.path.dirname(values["destination"])
        )
        if os.path.basename(owner) != "lock-root":
            fail("lock provisioner owner is not the exact lock-root")
        phase = mode
    elif mode in ("qualification-payload", "qualification-source"):
        owner = (
            values["destination_root"]
            if mode == "qualification-payload"
            else os.path.dirname(values["destination"])
        )
        match = re.fullmatch(r"/private/tmp/engram-c2b2a-([AB])-[A-Za-z0-9._-]+", owner)
        if match is None:
            fail("qualification root does not match the frozen A/B grammar")
        phase = "qualification-" + match.group(1) + "-" + mode.split("-", 1)[1]
    else:
        fail("unknown provisioner mode")
    return owner, phase


def require_exact_environment(mode: str, values: Mapping[str, str]) -> tuple[str, str]:
    owner, phase = phase_owner(mode, values)
    home = owner + "/home"
    if mode.startswith("qualification-"):
        home += "/active"
    expected = {
        "LANG": "C",
        "LC_ALL": "C",
        "TZ": "UTC",
        "SOURCE_DATE_EPOCH": "0",
        "__CF_USER_TEXT_ENCODING": "0x1F6:0x0:0x0",
        "HOME": home,
        "TMPDIR": owner + "/tmp/" + phase,
    }
    if dict(os.environ) != expected:
        fail("provisioner environment is not the exact seven-entry phase environment")
    if os.getcwd() != "/private/var/empty":
        fail("provisioner physical current directory is not the frozen empty directory")
    cwd_fd = os.open(".", os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC)
    empty_fd = os.open(
        "/private/var/empty",
        os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
    )
    try:
        cwd_info = require_traversable_directory_fd(cwd_fd)
        empty_info = require_traversable_directory_fd(empty_fd)
        require_absolute_metadata("/private/var/empty", cwd_fd, cwd_info, None)
        require_absolute_metadata("/private/var/empty", empty_fd, empty_info, None)
        if (cwd_info.st_dev, cwd_info.st_ino) != (empty_info.st_dev, empty_info.st_ino):
            fail("provisioner cwd is not the physical /var/empty object")
    finally:
        os.close(cwd_fd)
        os.close(empty_fd)
    if (
        os.getuid() != 502
        or os.geteuid() != 502
        or os.getgid() != 20
        or os.getegid() != 20
    ):
        fail("invoking identity is not the frozen unprivileged identity")
    previous = os.umask(0o077)
    os.umask(previous)
    if previous != 0o077:
        fail("process umask is not exactly 0077")
    return owner, phase


def open_private_tmp_parent() -> tuple[int, tuple[object, ...]]:
    root_fd = os.open("/", os.O_RDONLY | O_DIRECTORY | O_CLOEXEC)
    try:
        root_info = require_traversable_directory_fd(root_fd)
        require_absolute_metadata("/", root_fd, root_info, None)
        private_fd = os.open(
            "private",
            os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
            dir_fd=root_fd,
        )
    except BaseException:
        os.close(root_fd)
        raise
    os.close(root_fd)
    try:
        private_info = require_traversable_directory_fd(private_fd)
        require_absolute_metadata("/private", private_fd, private_info, None)
        tmp_fd = os.open(
            "tmp",
            os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
            dir_fd=private_fd,
        )
    except BaseException:
        os.close(private_fd)
        raise
    os.close(private_fd)
    try:
        _, stable = require_private_tmp_parent_fd(tmp_fd)
        return tmp_fd, stable
    except BaseException:
        os.close(tmp_fd)
        raise


def require_direct_root_postcondition(parent_fd: int, direct_root: str) -> None:
    if os.path.dirname(direct_root) != "/private/tmp":
        fail("controller-normalized root is not a direct /private/tmp child")
    leaf = os.path.basename(direct_root)
    reject_controls(leaf, allow_slash=False)
    listed = os.stat(leaf, dir_fd=parent_fd, follow_symlinks=False)
    fd = os.open(
        leaf,
        os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
        dir_fd=parent_fd,
    )
    try:
        opened = require_directory_fd(
            fd,
            mode=0o700,
            object_class="generated",
        )
        if not stable_identity(listed, opened):
            fail("controller-normalized root changed while opening")
        relisted = os.stat(leaf, dir_fd=parent_fd, follow_symlinks=False)
        if not stable_identity(opened, relisted):
            fail("controller-normalized root lookup changed during validation")
    finally:
        os.close(fd)


def require_phase_layout(mode: str, values: Mapping[str, str], owner: str, phase: str) -> None:
    parent_fd, parent_before = open_private_tmp_parent()
    try:
        direct_root = os.path.dirname(owner) if mode in ("lock-payload", "lock-source") else owner
        require_direct_root_postcondition(parent_fd, direct_root)
        validate_absolute(owner)
        owner_fd = open_directory_absolute(owner)
        try:
            owner_info = require_directory_fd(
                owner_fd,
                mode=0o700,
                object_class="generated",
            )
            bind_owner(owner, owner_info)
        finally:
            os.close(owner_fd)
        _, parent_after = require_private_tmp_parent_fd(parent_fd)
        if parent_before != parent_after:
            fail("stable /private/tmp parent metadata changed during phase-layout validation")
    finally:
        os.close(parent_fd)
    if mode.startswith("qualification-"):
        for kind in ("home", "cargo-home"):
            parent_fd = open_directory_absolute(owner + "/" + kind)
            try:
                require_directory_fd(
                    parent_fd,
                    mode=0o700,
                    object_class="generated",
                )
            finally:
                os.close(parent_fd)
            active_fd = open_directory_absolute(owner + "/" + kind + "/active")
            try:
                require_directory_fd(
                    active_fd,
                    mode=0o700,
                    empty=True,
                    object_class="generated",
                )
            finally:
                os.close(active_fd)
    else:
        home_fd = open_directory_absolute(owner + "/home")
        try:
            require_directory_fd(
                home_fd,
                mode=0o700,
                object_class="generated",
            )
        finally:
            os.close(home_fd)
    tmp_parent_fd = open_directory_absolute(owner + "/tmp")
    try:
        require_directory_fd(
            tmp_parent_fd,
            mode=0o700,
            object_class="generated",
        )
    finally:
        os.close(tmp_parent_fd)
    tmp = owner + "/tmp/" + phase
    tmp_fd = open_directory_absolute(tmp)
    try:
        require_directory_fd(
            tmp_fd,
            mode=0o700,
            empty=True,
            object_class="generated",
        )
    finally:
        os.close(tmp_fd)
    if mode == "acquisition-stage":
        if values["archive_staging"] != owner + "/archive-staging":
            fail("acquisition staging output path is not exact")
        if values["manifest_authority"] != owner + "/manifest-authority":
            fail("manifest authority output path is not exact")
        if values["lockfile"] != owner + "/seed-payload/Cargo.lock":
            fail("acquisition lockfile path is not exact")
        if not values["archive_dir"].startswith(owner + "/cargo-home/"):
            fail("acquisition archive cache is outside the fresh Cargo home")
    elif mode == "lock-payload":
        if values["destination_root"] != owner:
            fail("lock payload destination root is not exact")
    elif mode == "lock-source":
        if values["destination"] != owner + "/cargo-source":
            fail("lock source destination is not exact")
    elif mode == "archive-seal":
        if values["destination"] != owner + "/sealed-archives":
            fail("sealed archive destination is not exact")
        if values["archive_dir"] != owner + "/archive-staging":
            fail("archive seal input is not the exact staging directory")
    elif mode == "qualification-payload":
        if values["destination_root"] != owner:
            fail("qualification payload destination root is not exact")
    elif mode == "qualification-source":
        if values["destination"] != owner + "/cargo-source":
            fail("qualification source destination is not exact")


def require_absent(path: str) -> None:
    validate_absolute(path, leaf_may_be_absent=True)
    parent, leaf = os.path.split(path)
    parent_fd = open_directory_absolute(parent)
    try:
        try:
            os.stat(leaf, dir_fd=parent_fd, follow_symlinks=False)
        except FileNotFoundError:
            return
        fail("required create-new durable output already exists")
    finally:
        os.close(parent_fd)


def require_owner_grammar(mode: str, owner: str) -> None:
    if mode in ("acquisition-stage", "archive-seal"):
        require_acquisition_root(owner)
    elif mode in ("lock-payload", "lock-source"):
        require_acquisition_root(os.path.dirname(owner))
        if owner != os.path.dirname(owner) + "/lock-root":
            fail("lock-root owner path is not exact")
    elif re.fullmatch(r"/private/tmp/engram-c2b2a-[AB]-[A-Za-z0-9._-]+", owner) is None:
        fail("qualification owner path is not exact")


def mkdir_new_at(parent_fd: int, name: str, mode: int = 0o700) -> int:
    reject_controls(name, allow_slash=False)
    try:
        os.mkdir(name, mode, dir_fd=parent_fd)
        fd = os.open(name, os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC, dir_fd=parent_fd)
    except OSError as exc:
        raise ProvisionError("create-new directory failed") from exc
    try:
        created = require_directory_fd(
            fd,
            mode=mode,
            empty=True,
            object_class="generated",
        )
        require_created_entry(parent_fd, name, created, directory=True)
        os.fsync(fd)
        return fd
    except BaseException:
        os.close(fd)
        raise


def mkdir_absolute_new(path: str) -> int:
    parent_fd, leaf = split_new_path(path)
    try:
        return mkdir_new_at(parent_fd, leaf)
    finally:
        os.close(parent_fd)


def create_regular_at(parent_fd: int, name: str) -> int:
    reject_controls(name, allow_slash=False)
    flags = os.O_RDWR | os.O_CREAT | os.O_EXCL | O_NOFOLLOW | O_CLOEXEC
    try:
        fd = os.open(name, flags, 0o600, dir_fd=parent_fd)
    except OSError as exc:
        raise ProvisionError("create-new regular file failed") from exc
    try:
        info = require_regular_fd(fd, object_class="generated")
        if stat.S_IMODE(info.st_mode) != 0o600 or info.st_size != 0:
            fail("new regular file does not have exact initial state")
        require_created_entry(parent_fd, name, info, directory=False)
        return fd
    except BaseException:
        os.close(fd)
        raise


def write_all(fd: int, data: bytes) -> None:
    view = memoryview(data)
    while view:
        written = os.write(fd, view)
        if written <= 0:
            fail("short write while creating output")
        view = view[written:]


def create_bytes_at(parent_fd: int, name: str, data: bytes) -> tuple[int, int, str]:
    fd = create_regular_at(parent_fd, name)
    try:
        write_all(fd, data)
        os.fsync(fd)
        info = require_regular_fd(fd, object_class="generated")
        if info.st_size != len(data):
            fail("created output size is not exact")
        size, digest = hash_open_file(fd, info, object_class="generated")
        if size != len(data) or digest != hashlib.sha256(data).hexdigest():
            fail("created output bytes are not exact")
        return info.st_dev, info.st_ino, digest
    finally:
        os.close(fd)


def copy_open_to_new(
    source_fd: int,
    source_before: os.stat_result,
    destination_parent_fd: int,
    destination_name: str,
    *,
    source_object_class: str,
    maximum: int = U64_MAX,
) -> tuple[int, str, tuple[int, int]]:
    require_regular_fd(
        source_fd,
        maximum=maximum,
        object_class=source_object_class,
    )
    os.lseek(source_fd, 0, os.SEEK_SET)
    destination_fd = create_regular_at(destination_parent_fd, destination_name)
    digest = hashlib.sha256()
    size = 0
    try:
        while True:
            block = os.read(source_fd, READ_CHUNK)
            if not block:
                break
            size = checked_add(size, len(block), maximum)
            digest.update(block)
            write_all(destination_fd, block)
        os.fsync(destination_fd)
        destination = require_regular_fd(
            destination_fd,
            maximum=maximum,
            object_class="generated",
        )
        if destination.st_size != size:
            fail("destination size differs from copied byte stream")
        destination_size, destination_digest = hash_open_file(
            destination_fd,
            destination,
            object_class="generated",
        )
        if destination_size != size or destination_digest != digest.hexdigest():
            fail("destination bytes differ from copied byte stream")
        source_after = require_regular_fd(
            source_fd,
            maximum=maximum,
            object_class=source_object_class,
        )
        if not stable_identity(source_before, source_after) or size != source_before.st_size:
            fail("source changed during descriptor-relative copy")
        if (source_before.st_dev, source_before.st_ino) == (
            destination.st_dev,
            destination.st_ino,
        ):
            fail("source and destination share an inode")
        return destination_size, destination_digest, (destination.st_dev, destination.st_ino)
    finally:
        os.close(destination_fd)


def open_regular_at(
    parent_fd: int,
    name: str,
    *,
    object_class: str,
    maximum: int = U64_MAX,
) -> tuple[int, os.stat_result]:
    reject_controls(name, allow_slash=False)
    try:
        fd = os.open(name, os.O_RDONLY | O_NOFOLLOW | O_CLOEXEC, dir_fd=parent_fd)
    except OSError as exc:
        raise ProvisionError("descriptor-relative regular input open failed") from exc
    try:
        info = require_regular_fd(fd, maximum=maximum, object_class=object_class)
        return fd, info
    except BaseException:
        os.close(fd)
        raise


def open_directory_at(
    parent_fd: int,
    name: str,
    *,
    object_class: str,
) -> int:
    reject_controls(name, allow_slash=False)
    try:
        fd = os.open(name, os.O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC, dir_fd=parent_fd)
    except OSError as exc:
        raise ProvisionError("descriptor-relative directory input open failed") from exc
    try:
        require_directory_fd(fd, object_class=object_class)
        return fd
    except BaseException:
        os.close(fd)
        raise


def sorted_entries(directory_fd: int) -> list[str]:
    try:
        names = os.listdir(directory_fd)
    except OSError as exc:
        raise ProvisionError("directory enumeration failed") from exc
    folded: dict[str, str] = {}
    normalized: dict[str, str] = {}
    for name in names:
        reject_controls(name, allow_slash=False)
        casefolded = name.casefold()
        nfc = unicodedata.normalize("NFC", name)
        if casefolded in folded and folded[casefolded] != name:
            fail("directory contains a default-case-folded name collision")
        if nfc in normalized and normalized[nfc] != name:
            fail("directory contains an NFC-normalized name collision")
        folded[casefolded] = name
        normalized[nfc] = name
    return sorted(names, key=raw_key)


def require_created_entry(
    parent_fd: int,
    name: str,
    opened: os.stat_result,
    *,
    directory: bool,
) -> None:
    names = sorted_entries(parent_fd)
    if names.count(name) != 1:
        fail("create-new entry spelling is not exact")
    listed = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    if (listed.st_dev, listed.st_ino) != (opened.st_dev, opened.st_ino):
        fail("create-new lookup does not name the opened inode")
    if directory != stat.S_ISDIR(listed.st_mode):
        fail("create-new entry type is not exact")


def seal_regular_fd(fd: int, executable: bool) -> None:
    before = require_regular_fd(fd, object_class="generated")
    os.fchmod(fd, 0o555 if executable else 0o444)
    os.utime(fd, ns=(0, 0))
    os.fsync(fd)
    after = require_regular_fd(fd, object_class="generated")
    if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
        fail("regular-file seal changed inode identity")
    if stat.S_IMODE(after.st_mode) != (0o555 if executable else 0o444):
        fail("regular-file seal mode is not exact")
    if after.st_mtime_ns != 0:
        fail("regular-file seal mtime is not epoch zero")


def seal_directory_fd(fd: int, mode: int) -> None:
    before = require_directory_fd(fd, object_class="generated")
    os.fchmod(fd, mode)
    os.utime(fd, ns=(0, 0))
    os.fsync(fd)
    after = require_directory_fd(fd, object_class="generated")
    if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
        fail("directory seal changed inode identity")
    if stat.S_IMODE(after.st_mode) != mode or after.st_mtime_ns != 0:
        fail("directory seal state is not exact")


def seal_tree(
    directory_fd: int,
    executable_paths: Mapping[str, bool],
    prefix: str = "",
    writable_relative: str | None = None,
) -> None:
    directory_before = require_directory_fd(
        directory_fd,
        object_class="generated",
    )
    names = sorted_entries(directory_fd)
    for name in names:
        relative = name if not prefix else prefix + "/" + name
        info = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
        if stat.S_ISDIR(info.st_mode):
            child_fd = open_directory_at(
                directory_fd,
                name,
                object_class="generated",
            )
            try:
                if not stable_identity(
                    info,
                    require_directory_fd(child_fd, object_class="generated"),
                ):
                    fail("directory identity changed before descendant sealing")
                seal_tree(child_fd, executable_paths, relative, writable_relative)
                seal_directory_fd(child_fd, 0o555)
            finally:
                os.close(child_fd)
        elif stat.S_ISREG(info.st_mode):
            child_fd, opened = open_regular_at(
                directory_fd,
                name,
                object_class="generated",
            )
            try:
                if not stable_identity(info, opened):
                    fail("regular-file identity changed before sealing")
                if relative not in executable_paths:
                    fail("seal metadata lacks a regular-file path")
                if relative == writable_relative:
                    before = require_regular_fd(
                        child_fd,
                        object_class="generated",
                    )
                    os.fchmod(child_fd, 0o600)
                    os.utime(child_fd, ns=(0, 0))
                    os.fsync(child_fd)
                    after = require_regular_fd(
                        child_fd,
                        object_class="generated",
                    )
                    if (
                        (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino)
                        or stat.S_IMODE(after.st_mode) != 0o600
                        or after.st_mtime_ns != 0
                    ):
                        fail("temporary writable Cargo.lock state is not exact")
                else:
                    seal_regular_fd(child_fd, executable_paths[relative])
            finally:
                os.close(child_fd)
        else:
            fail("tree contains a non-regular entry")
    directory_after = require_directory_fd(
        directory_fd,
        object_class="generated",
    )
    if not stable_identity(directory_before, directory_after) or names != sorted_entries(
        directory_fd
    ):
        fail("directory changed while its descendants were sealed")


def copy_tree(
    source_fd: int,
    destination_fd: int,
    *,
    source_object_class: str,
    prefix: str = "",
    executable_paths: dict[str, bool] | None = None,
    destination_inodes: set[tuple[int, int]] | None = None,
) -> tuple[dict[str, bool], set[tuple[int, int]]]:
    if executable_paths is None:
        executable_paths = {}
    if destination_inodes is None:
        destination_inodes = set()
    directory_before = require_directory_fd(
        source_fd,
        object_class=source_object_class,
    )
    names_before = sorted_entries(source_fd)
    for name in names_before:
        relative = name if not prefix else prefix + "/" + name
        validate_relative(relative)
        info = os.stat(name, dir_fd=source_fd, follow_symlinks=False)
        if stat.S_ISDIR(info.st_mode):
            child_source = open_directory_at(
                source_fd,
                name,
                object_class=source_object_class,
            )
            child_destination = mkdir_new_at(destination_fd, name)
            try:
                if not stable_identity(
                    info,
                    require_directory_fd(
                        child_source,
                        object_class=source_object_class,
                    ),
                ):
                    fail("source directory identity changed between listing and open")
                copy_tree(
                    child_source,
                    child_destination,
                    prefix=relative,
                    source_object_class=source_object_class,
                    executable_paths=executable_paths,
                    destination_inodes=destination_inodes,
                )
            finally:
                os.close(child_source)
                os.close(child_destination)
        elif stat.S_ISREG(info.st_mode):
            source_file, file_before = open_regular_at(
                source_fd,
                name,
                object_class=source_object_class,
            )
            try:
                if not stable_identity(info, file_before):
                    fail("source file identity changed between listing and open")
                _, _, inode = copy_open_to_new(
                    source_file,
                    file_before,
                    destination_fd,
                    name,
                    source_object_class=source_object_class,
                )
            finally:
                os.close(source_file)
            if inode in destination_inodes:
                fail("copied payload files share an inode")
            destination_inodes.add(inode)
            executable_paths[relative] = bool(file_before.st_mode & 0o111)
        else:
            fail("source tree contains a link or special entry")
    source_after = require_directory_fd(
        source_fd,
        object_class=source_object_class,
    )
    if (
        not stable_identity(directory_before, source_after)
        or names_before != sorted_entries(source_fd)
    ):
        fail("source directory changed during recursive copy")
    return executable_paths, destination_inodes


def replace_relative_file(root_fd: int, relative: str, expected: bytes, replacement: bytes) -> None:
    parts = validate_relative(relative)
    parent_fd = os.dup(root_fd)
    try:
        for part in parts[:-1]:
            child = open_directory_at(
                parent_fd,
                part,
                object_class="generated",
            )
            os.close(parent_fd)
            parent_fd = child
        fd = os.open(parts[-1], os.O_RDWR | O_NOFOLLOW | O_CLOEXEC, dir_fd=parent_fd)
        try:
            before = require_regular_fd(
                fd,
                maximum=FILE_BYTES_MAX,
                object_class="generated",
            )
            current, current_info = read_open_file(
                fd,
                FILE_BYTES_MAX,
                object_class="generated",
            )
            if not stable_identity(before, current_info) or current != expected:
                fail("patch old context does not match the exact file bytes")
            os.lseek(fd, 0, os.SEEK_SET)
            os.ftruncate(fd, 0)
            write_all(fd, replacement)
            os.fsync(fd)
            after = require_regular_fd(
                fd,
                maximum=FILE_BYTES_MAX,
                object_class="generated",
            )
            if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
                fail("patched file identity changed")
            size, digest = hash_open_file(
                fd,
                after,
                object_class="generated",
            )
            if size != len(replacement) or digest != hashlib.sha256(replacement).hexdigest():
                fail("patched file output bytes are not exact")
        finally:
            os.close(fd)
    finally:
        os.close(parent_fd)


@dataclass(frozen=True)
class InventoryFile:
    size: int
    sha256: str
    executable: bool
    inode: tuple[int, int]


def inventory_tree(
    root_fd: int,
    *,
    object_class: str,
) -> tuple[set[str], dict[str, InventoryFile]]:
    directories: set[str] = set()
    files: dict[str, InventoryFile] = {}
    inodes: set[tuple[int, int]] = set()

    def walk(directory_fd: int, prefix: str) -> None:
        before = require_directory_fd(directory_fd, object_class=object_class)
        names = sorted_entries(directory_fd)
        for name in names:
            relative = name if not prefix else prefix + "/" + name
            validate_relative(relative)
            listed = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
            if stat.S_ISDIR(listed.st_mode):
                child = open_directory_at(
                    directory_fd,
                    name,
                    object_class=object_class,
                )
                try:
                    opened = require_directory_fd(child, object_class=object_class)
                    if not stable_identity(listed, opened):
                        fail("directory identity changed between listing and open")
                    directories.add(relative)
                    walk(child, relative)
                finally:
                    os.close(child)
            elif stat.S_ISREG(listed.st_mode):
                child, opened = open_regular_at(
                    directory_fd,
                    name,
                    object_class=object_class,
                )
                try:
                    if not stable_identity(listed, opened):
                        fail("file identity changed between listing and open")
                    size, digest = hash_open_file(
                        child,
                        opened,
                        object_class=object_class,
                    )
                finally:
                    os.close(child)
                inode = (opened.st_dev, opened.st_ino)
                if inode in inodes:
                    fail("tree contains shared regular-file inodes")
                inodes.add(inode)
                files[relative] = InventoryFile(size, digest, bool(opened.st_mode & 0o111), inode)
            else:
                fail("tree contains a link or special entry")
        after = require_directory_fd(directory_fd, object_class=object_class)
        if not stable_identity(before, after) or names != sorted_entries(directory_fd):
            fail("directory changed during inventory")

    walk(root_fd, "")
    return directories, files


def encode_xattr_sidecar(
    objects: Sequence[tuple[str, str, tuple[tuple[bytes, bytes], ...]]],
) -> tuple[bytes, int]:
    if not objects or objects[0][1] != ".":
        fail("xattr sidecar does not begin with its selected root")
    paths = [path for _, path, _ in objects]
    if len(paths) != len(set(paths)):
        fail("xattr sidecar contains a duplicate object path")
    if paths[1:] != sorted(paths[1:], key=raw_key):
        fail("xattr sidecar object paths are not in raw UTF-8 order")
    chunks = [PAYLOAD_XATTR_SCHEMA]
    stream_bytes = len(PAYLOAD_XATTR_SCHEMA)
    rows = 1
    value_bytes = 0
    for kind, path, attributes in objects:
        if kind not in ("directory", "file", "symlink"):
            fail("xattr sidecar object kind is invalid")
        if path != ".":
            validate_relative(path)
        kind_bytes = kind.encode("ascii")
        path_bytes = raw_key(path)
        object_row_bytes = len(b"object\t") + len(kind_bytes) + 1 + len(path_bytes) + 1
        rows = checked_add(rows, 1, XATTR_SIDECAR_ROWS_MAX)
        stream_bytes = checked_add(
            stream_bytes,
            object_row_bytes,
            XATTR_SIDECAR_BYTES_MAX,
        )
        object_row = b"object\t" + kind_bytes + b"\t" + path_bytes + b"\n"
        if len(object_row) != object_row_bytes:
            fail("xattr sidecar object-row byte accounting is inconsistent")
        chunks.append(object_row)
        names = tuple(name for name, _ in attributes)
        if names != tuple(sorted(names)) or len(names) != len(set(names)):
            fail("xattr sidecar attribute rows are duplicated or reordered")
        for name, value in attributes:
            if not name or len(name) > 255 or b"\x00" in name:
                fail("xattr sidecar attribute name is invalid")
            if len(value) > SYSTEM_XATTR_VALUE_MAX:
                fail("xattr sidecar attribute value exceeds its frozen bound")
            value_bytes = checked_add(
                value_bytes,
                len(value),
                SYSTEM_XATTR_VALUES_TOTAL_MAX,
            )
            value_hex_bytes = 1 if not value else 2 * len(value)
            attribute_row_bytes = (
                len(b"xattr\t")
                + len(str(len(name)).encode("ascii"))
                + 1
                + 64
                + 1
                + 2 * len(name)
                + 1
                + len(str(len(value)).encode("ascii"))
                + 1
                + 64
                + 1
                + value_hex_bytes
                + 1
            )
            rows = checked_add(rows, 1, XATTR_SIDECAR_ROWS_MAX)
            stream_bytes = checked_add(
                stream_bytes,
                attribute_row_bytes,
                XATTR_SIDECAR_BYTES_MAX,
            )
            attribute_row = (
                b"xattr\t"
                + str(len(name)).encode("ascii")
                + b"\t"
                + hashlib.sha256(name).hexdigest().encode("ascii")
                + b"\t"
                + name.hex().encode("ascii")
                + b"\t"
                + str(len(value)).encode("ascii")
                + b"\t"
                + hashlib.sha256(value).hexdigest().encode("ascii")
                + b"\t"
                + (value.hex().encode("ascii") if value else b"-")
                + b"\n"
            )
            if len(attribute_row) != attribute_row_bytes:
                fail("xattr sidecar attribute-row byte accounting is inconsistent")
            chunks.append(attribute_row)
    stream = b"".join(chunks)
    if len(stream) != stream_bytes:
        fail("xattr sidecar byte accounting is inconsistent")
    return stream, rows


def expected_payload_xattr_sidecar(manifest_present: bool) -> tuple[bytes, bool]:
    accepted_manifest_present, sidecar_identity = provisioner_payload_sidecar_identity(
        manifest_present
    )
    manifest_present = None
    unordered_paths = list(PAYLOAD_XATTR_OBJECTS) + [
        ("file", "build-support/archive-manifest-v1.tsv")
    ] * int(accepted_manifest_present)
    root = unordered_paths[0]
    paths = [root] + sorted(unordered_paths[1:], key=lambda row: raw_key(row[1]))
    objects = tuple((kind, path, PROVENANCE_FRESH_ROW) for kind, path in paths)
    stream, rows = encode_xattr_sidecar(objects)
    if sidecar_identity not in (
        PAYLOAD_XATTR_CURRENT_IDENTITY,
        PAYLOAD_XATTR_MANIFEST_IDENTITY,
    ):
        fail("embedded payload xattr sidecar authority is internally inconsistent")
    try:
        accepted_stream = provisioner_accept_payload_sidecar(
            stream,
            len(objects),
            rows,
            sidecar_identity,
        )
    except ValueError as exc:
        raise ProvisionError(
            "embedded payload xattr sidecar authority is internally inconsistent"
        ) from exc
    return accepted_stream, accepted_manifest_present


def collect_payload_xattr_sidecar(
    root_fd: int,
    *,
    object_class: str,
) -> bytes:
    objects: list[tuple[str, str, tuple[tuple[bytes, bytes], ...]]] = [
        ("directory", ".", xattrs_fd(root_fd, system_input=False))
    ]

    def walk(directory_fd: int, prefix: str) -> None:
        before = require_directory_fd(directory_fd, object_class=object_class)
        names = sorted_entries(directory_fd)
        for name in names:
            relative = name if not prefix else prefix + "/" + name
            validate_relative(relative)
            listed = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
            if stat.S_ISDIR(listed.st_mode):
                child = open_directory_at(
                    directory_fd,
                    name,
                    object_class=object_class,
                )
                try:
                    opened = require_directory_fd(child, object_class=object_class)
                    if not stable_identity(listed, opened):
                        fail("xattr sidecar directory changed while opening")
                    objects.append(
                        (
                            "directory",
                            relative,
                            xattrs_fd(child, system_input=False),
                        )
                    )
                    walk(child, relative)
                finally:
                    os.close(child)
            elif stat.S_ISREG(listed.st_mode):
                child, opened = open_regular_at(
                    directory_fd,
                    name,
                    object_class=object_class,
                )
                try:
                    if not stable_identity(listed, opened):
                        fail("xattr sidecar regular file changed while opening")
                    objects.append(
                        ("file", relative, xattrs_fd(child, system_input=False))
                    )
                finally:
                    os.close(child)
            else:
                fail("payload xattr sidecar encountered an unsupported object type")
        after = require_directory_fd(directory_fd, object_class=object_class)
        if not stable_identity(before, after) or names != sorted_entries(directory_fd):
            fail("payload changed while collecting its xattr sidecar")

    walk(root_fd, "")
    root = objects[0]
    ordered = [root] + sorted(objects[1:], key=lambda row: raw_key(row[1]))
    stream, _ = encode_xattr_sidecar(ordered)
    return stream


def require_payload_xattr_sidecar(
    root_fd: int,
    manifest_present: bool,
    *,
    object_class: str,
) -> tuple[bytes, bool]:
    observed = collect_payload_xattr_sidecar(
        root_fd,
        object_class=object_class,
    )
    expected, accepted_manifest_present = expected_payload_xattr_sidecar(manifest_present)
    manifest_present = None
    if observed != expected:
        fail("payload xattr sidecar differs from the complete semantic authority")
    return expected, accepted_manifest_present


def require_bytes_in_tree(
    root_fd: int,
    relative: str,
    expected: bytes,
    *,
    object_class: str,
) -> None:
    parts = validate_relative(relative)
    parent_fd = os.dup(root_fd)
    try:
        for part in parts[:-1]:
            child = open_directory_at(parent_fd, part, object_class=object_class)
            os.close(parent_fd)
            parent_fd = child
        child, _ = open_regular_at(
            parent_fd,
            parts[-1],
            maximum=max(len(expected), 1),
            object_class=object_class,
        )
        try:
            actual, _ = read_open_file(
                child,
                max(len(expected), 1),
                object_class=object_class,
            )
        finally:
            os.close(child)
    finally:
        os.close(parent_fd)
    if actual != expected:
        fail("payload support or configuration bytes are not exact")


def read_bytes_in_tree(
    root_fd: int,
    relative: str,
    maximum: int,
    *,
    object_class: str,
) -> bytes:
    parts = validate_relative(relative)
    parent_fd = os.dup(root_fd)
    try:
        for part in parts[:-1]:
            child = open_directory_at(parent_fd, part, object_class=object_class)
            os.close(parent_fd)
            parent_fd = child
        child, _ = open_regular_at(
            parent_fd,
            parts[-1],
            maximum=maximum,
            object_class=object_class,
        )
        try:
            data, _ = read_open_file(child, maximum, object_class=object_class)
        finally:
            os.close(child)
        return data
    finally:
        os.close(parent_fd)


def validate_contract_test(root_fd: int, *, object_class: str) -> None:
    data = read_bytes_in_tree(
        root_fd,
        "tests/contract.rs",
        64 * 1024,
        object_class=object_class,
    )
    if data.count(CONTRACT_DIRECT_FINAL + CONTRACT_MANIFEST_ASSERTIONS) != 1:
        fail("contract test direct dependency assertion is not the exact frozen edit")
    if data.count(CONTRACT_MANIFEST_ASSERTIONS) != 1:
        fail("contract test manifest assertions are not the exact frozen edit")
    final_lock_prefix = CONTRACT_LOCK_PREFIX + CONTRACT_LOCK_TUPLE + CONTRACT_LOCK_FIRST_TUPLE
    if data.count(CONTRACT_LOCK_TUPLE) != 1 or data.count(final_lock_prefix) != 1:
        fail("contract test lock tuple assertion is not the exact frozen edit")
    reconstructed = data.replace(CONTRACT_DIRECT_FINAL, CONTRACT_DIRECT_PRE, 1)
    reconstructed = reconstructed.replace(CONTRACT_MANIFEST_ASSERTIONS, b"", 1)
    reconstructed = reconstructed.replace(
        final_lock_prefix,
        CONTRACT_LOCK_PREFIX + CONTRACT_LOCK_FIRST_TUPLE,
        1,
    )
    if (
        len(reconstructed) != CONTRACT_PRE_SIZE
        or hashlib.sha256(reconstructed).hexdigest() != CONTRACT_PRE_SHA256
    ):
        fail("contract test contains a change outside the exact reversible delta")


def validate_lock(
    root_fd: int,
    manifest_expected: bytes | None,
    *,
    accepted_manifest_present: bool,
    final_lock: bool,
    object_class: str,
) -> None:
    data = read_bytes_in_tree(
        root_fd,
        "Cargo.lock",
        MANIFEST_BYTES_MAX * 4,
        object_class=object_class,
    )
    packages = parse_lock_bytes(data)
    try:
        parsed = tomllib.loads(decode_utf8(data, "payload Cargo.lock"))
    except tomllib.TOMLDecodeError as exc:
        raise ProvisionError("payload Cargo.lock is invalid") from exc
    roots = [
        item
        for item in parsed["package"]
        if isinstance(item, dict) and item.get("source") is None
    ]
    if len(roots) != 1 or (roots[0].get("name"), roots[0].get("version")) != ROOT_PACKAGE:
        fail("payload Cargo.lock path root is not exact")
    if accepted_manifest_present:
        rows = parse_manifest_bytes(manifest_expected)
        compare_manifest_to_lock(rows, packages)
    if not final_lock:
        if (
            len(data) != 108_142
            or hashlib.sha256(data).hexdigest()
            != "40a484a61cd15e2555e0a18bd532a8229d78ab4144945b2fe56984c271e7f19b"
        ):
            fail("lock-payload source does not retain the exact pre-repair lock")
        expected_dependencies = [
            "getrandom 0.3.4",
            "libc",
            "sha2",
            "surrealdb-core",
            "tokio",
        ]
    else:
        if data.count(LOCK_ROOT_FINAL) != 1:
            fail("final Cargo.lock does not contain the one exact added root edge")
        reconstructed = data.replace(LOCK_ROOT_FINAL, LOCK_ROOT_PRE, 1)
        if (
            len(reconstructed) != 108_142
            or hashlib.sha256(reconstructed).hexdigest()
            != "40a484a61cd15e2555e0a18bd532a8229d78ab4144945b2fe56984c271e7f19b"
        ):
            fail("final Cargo.lock differs outside the exact reversible root-edge delta")
        expected_dependencies = [
            "getrandom 0.2.17",
            "getrandom 0.3.4",
            "libc",
            "sha2",
            "surrealdb-core",
            "tokio",
        ]
    if roots[0].get("dependencies") != expected_dependencies:
        fail("payload Cargo.lock root dependency edge set is not exact")


def validate_cargo_manifest(root_fd: int, *, object_class: str) -> None:
    parts = ("Cargo.toml",)
    fd, _ = open_regular_at(
        root_fd,
        parts[0],
        maximum=16_384,
        object_class=object_class,
    )
    try:
        data, _ = read_open_file(fd, 16_384, object_class=object_class)
    finally:
        os.close(fd)
    if data != CARGO_TOML_BYTES:
        fail("payload Cargo.toml differs outside the exact deterministic edit")
    try:
        parsed = tomllib.loads(decode_utf8(data, "payload Cargo.toml"))
    except tomllib.TOMLDecodeError as exc:
        raise ProvisionError("payload Cargo.toml is invalid") from exc
    package = parsed.get("package")
    if (
        not isinstance(package, dict)
        or (package.get("name"), package.get("version")) != ROOT_PACKAGE
    ):
        fail("payload Cargo.toml root package identity is not exact")
    features = parsed.get("features")
    if not isinstance(features, dict) or features.get("collector") != [
        "dep:getrandom02",
        "dep:surrealdb-core",
        "dep:tokio",
    ]:
        fail("payload collector feature does not contain the exact alias edge")
    dependencies = parsed.get("dependencies")
    if not isinstance(dependencies, dict) or tuple(dependencies) != (
        "getrandom",
        "libc",
        "sha2",
        "getrandom02",
        "surrealdb-core",
        "tokio",
    ):
        fail("payload direct dependency names or order are not exact")
    alias = dependencies.get("getrandom02")
    if alias != {
        "package": "getrandom",
        "version": "=0.2.17",
        "default-features": False,
        "features": ["linux_disable_fallback"],
        "optional": True,
    }:
        fail("payload getrandom02 alias table is not exact")


def validate_payload(
    root_fd: int,
    *,
    manifest_expected: bytes | None,
    final_lock: bool,
    object_class: str,
) -> None:
    manifest_present = manifest_expected is not None
    sidecar_before, accepted_manifest_present = require_payload_xattr_sidecar(
        root_fd,
        manifest_present,
        object_class=object_class,
    )
    manifest_present = None
    if accepted_manifest_present != (manifest_expected is not None):
        fail("accepted payload manifest selector differs from manifest bytes")
    directories, files = inventory_tree(root_fd, object_class=object_class)
    expected_directories = set(BASELINE_DIRS)
    expected_directories.add("build-support")
    if directories != expected_directories:
        fail("payload directory set is outside the frozen structural boundary")
    expected_files = set(BASELINE_FILES) | CHANGED_FILES | SUPPORT_FILES
    if not accepted_manifest_present:
        expected_files.remove("build-support/archive-manifest-v1.tsv")
    if set(files) != expected_files:
        fail("payload file set is outside the frozen structural boundary")
    for path, (size, digest) in BASELINE_FILES.items():
        observed = files[path]
        if observed.size != size or observed.sha256 != digest or observed.executable:
            fail("unchanged frozen payload file differs from its baseline row")
    for path, observed in files.items():
        if observed.executable:
            fail("payload contains an unauthorized executable file")
    patch = files["patches/surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch"]
    if patch.size != PATCH_BYTES or patch.sha256 != PATCH_SHA256:
        fail("payload Rocks patch identity is not exact")
    require_bytes_in_tree(
        root_fd,
        ".cargo/config.toml",
        PAYLOAD_CONFIG_BYTES,
        object_class=object_class,
    )
    require_bytes_in_tree(
        root_fd,
        "build-support/root-cargo-config.toml",
        ROOT_CONFIG_BYTES,
        object_class=object_class,
    )
    if accepted_manifest_present:
        require_bytes_in_tree(
            root_fd,
            "build-support/archive-manifest-v1.tsv",
            manifest_expected,
            object_class=object_class,
        )
    validate_cargo_manifest(root_fd, object_class=object_class)
    validate_contract_test(root_fd, object_class=object_class)
    validate_lock(
        root_fd,
        manifest_expected,
        accepted_manifest_present=accepted_manifest_present,
        final_lock=final_lock,
        object_class=object_class,
    )
    sidecar_after, accepted_manifest_present_after = require_payload_xattr_sidecar(
        root_fd,
        accepted_manifest_present,
        object_class=object_class,
    )
    if (
        sidecar_before != sidecar_after
        or accepted_manifest_present != accepted_manifest_present_after
    ):
        fail("payload xattr sidecar changed during semantic validation")


def copy_payload_mode(
    payload_source: str,
    destination_root: str,
    root_config: str,
    manifest_path: str | None,
) -> None:
    source_fd = open_directory_absolute(payload_source)
    config_fd, config_info = open_regular_absolute(
        root_config,
        maximum=len(ROOT_CONFIG_BYTES),
        object_class="repository-fresh",
    )
    manifest_data: bytes | None = None
    if manifest_path is not None:
        manifest_data, _ = read_manifest(manifest_path)
    try:
        config_data, config_read_info = read_open_file(
            config_fd,
            len(ROOT_CONFIG_BYTES),
            object_class="repository-fresh",
        )
        if not stable_identity(config_info, config_read_info) or config_data != ROOT_CONFIG_BYTES:
            fail("root Cargo configuration authority is not exact")
        source_manifest = (
            None
            if manifest_path
            else read_manifest_from_payload(
                source_fd,
                object_class="repository-fresh",
            )
        )
        final_lock = manifest_path is None
        validate_payload(
            source_fd,
            manifest_expected=source_manifest,
            final_lock=final_lock,
            object_class="repository-fresh",
        )
        root_fd = open_directory_absolute(destination_root)
        try:
            payload_fd = mkdir_new_at(root_fd, "payload")
            cargo_fd = mkdir_new_at(root_fd, ".cargo")
            try:
                executable, _ = copy_tree(
                    source_fd,
                    payload_fd,
                    source_object_class="repository-fresh",
                )
                if manifest_data is not None:
                    support_fd = open_directory_at(
                        payload_fd,
                        "build-support",
                        object_class="generated",
                    )
                    try:
                        create_bytes_at(support_fd, "archive-manifest-v1.tsv", manifest_data)
                        executable["build-support/archive-manifest-v1.tsv"] = False
                    finally:
                        os.close(support_fd)
                _, copied_digest, _ = copy_open_to_new(
                    config_fd,
                    config_info,
                    cargo_fd,
                    "config.toml",
                    maximum=len(ROOT_CONFIG_BYTES),
                    source_object_class="repository-fresh",
                )
                if copied_digest != hashlib.sha256(ROOT_CONFIG_BYTES).hexdigest():
                    fail("copied root Cargo configuration digest differs")
                writable_lock = "Cargo.lock" if manifest_path is not None else None
                seal_tree(payload_fd, executable, writable_relative=writable_lock)
                seal_directory_fd(payload_fd, 0o555)
                config_copy, _ = open_regular_at(
                    cargo_fd,
                    "config.toml",
                    object_class="generated",
                )
                try:
                    seal_regular_fd(config_copy, False)
                finally:
                    os.close(config_copy)
                seal_directory_fd(cargo_fd, 0o555)
                validate_payload(
                    payload_fd,
                    manifest_expected=(
                        manifest_data if manifest_data is not None else source_manifest
                    ),
                    final_lock=final_lock,
                    object_class="generated",
                )
            finally:
                os.close(payload_fd)
                os.close(cargo_fd)
        finally:
            os.close(root_fd)
    finally:
        os.close(source_fd)
        os.close(config_fd)


def read_manifest_from_payload(payload_fd: int, *, object_class: str) -> bytes:
    support_fd = open_directory_at(
        payload_fd,
        "build-support",
        object_class=object_class,
    )
    try:
        manifest_fd, _ = open_regular_at(
            support_fd,
            "archive-manifest-v1.tsv",
            maximum=MANIFEST_BYTES_MAX,
            object_class=object_class,
        )
        try:
            data, _ = read_open_file(
                manifest_fd,
                MANIFEST_BYTES_MAX,
                object_class=object_class,
            )
        finally:
            os.close(manifest_fd)
    finally:
        os.close(support_fd)
    parse_manifest_bytes(data)
    return data


def is_beneath(path: str, root: str) -> bool:
    return path == root or path.startswith(root + "/")


def require_disjoint(*paths: str) -> None:
    for index, left in enumerate(paths):
        for right in paths[index + 1 :]:
            if is_beneath(left, right) or is_beneath(right, left):
                fail("input and output paths overlap")


def seal_archive_directory(directory_fd: int, manifest_name: str | None) -> None:
    directory_before = require_directory_fd(
        directory_fd,
        object_class="generated",
    )
    names = sorted_entries(directory_fd)
    for name in names:
        info = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
        if not stat.S_ISREG(info.st_mode):
            fail("archive set contains a non-regular entry")
        fd, opened = open_regular_at(
            directory_fd,
            name,
            maximum=ARCHIVE_BYTES_MAX,
            object_class="generated",
        )
        try:
            if not stable_identity(info, opened):
                fail("archive identity changed before sealing")
            before = require_regular_fd(fd, object_class="generated")
            os.fchmod(fd, 0o400)
            os.utime(fd, ns=(0, 0))
            os.fsync(fd)
            after = require_regular_fd(fd, object_class="generated")
            if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
                fail("archive seal changed inode identity")
            if stat.S_IMODE(after.st_mode) != 0o400 or after.st_mtime_ns != 0:
                fail("archive seal state is not exact")
        finally:
            os.close(fd)
    directory_after = require_directory_fd(
        directory_fd,
        object_class="generated",
    )
    if not stable_identity(directory_before, directory_after) or names != sorted_entries(
        directory_fd
    ):
        fail("archive directory changed while its files were sealed")
    if manifest_name is not None and manifest_name not in names:
        fail("sealed bundle is missing its manifest")
    seal_directory_fd(directory_fd, 0o500)


def require_acquisition_registry_marker(registry_fd: int) -> None:
    profile, private_tmp_parent, accepted_registry_fd = provisioner_object_profile(
        "acquisition-registry",
        registry_fd,
    )
    registry_fd = None
    before = require_directory_fd(
        accepted_registry_fd,
        object_class="acquisition-registry",
    )
    attributes = xattrs_fd(
        accepted_registry_fd,
        system_input=False,
        private_tmp_parent=private_tmp_parent,
    )
    profile_accepted = provisioner_profile_accepts(attributes, profile)
    if not profile_accepted:
        fail("acquisition registry xattrs differ from their exact closed profile")
    accepted_attributes = provisioner_expected_xattrs(profile)
    names = sorted_entries(accepted_registry_fd)
    if "CACHEDIR.TAG" not in names:
        fail("acquisition registry is missing its exact Cargo cache marker")
    tag_fd, tag_before = open_regular_at(
        accepted_registry_fd,
        "CACHEDIR.TAG",
        maximum=len(CACHEDIR_TAG_BYTES),
        object_class="cargo-ordinary",
    )
    try:
        require_owner_only_mode(tag_before)
        tag_bytes, tag_after = read_open_file(
            tag_fd,
            len(CACHEDIR_TAG_BYTES),
            object_class="cargo-ordinary",
        )
        if not stable_identity(tag_before, tag_after):
            fail("Cargo cache marker changed while reading")
        tag_size, tag_digest = hash_open_file(
            tag_fd,
            tag_after,
            object_class="cargo-ordinary",
        )
        if tag_bytes != CACHEDIR_TAG_BYTES:
            fail("Cargo cache marker bytes are not exact")
    finally:
        os.close(tag_fd)
    marker_accepted = provisioner_cargo_marker_accepts(
        "acquisition-registry",
        accepted_attributes,
        tag_size,
        tag_digest,
    )
    if not marker_accepted:
        fail("acquisition registry marker profile or tag identity is not exact")
    after = require_directory_fd(
        accepted_registry_fd,
        object_class="acquisition-registry",
    )
    if not stable_identity(before, after) or names != sorted_entries(accepted_registry_fd):
        fail("acquisition registry changed while validating its Cargo marker")


def open_acquisition_archive_directory(owner: str, archive_dir: str) -> int:
    cargo_home = owner + "/cargo-home"
    leaf = os.path.basename(archive_dir)
    if archive_dir != cargo_home + "/registry/cache/" + leaf:
        fail("fresh Cargo archive cache is not its exact owner-root descendant")
    directory_fd = open_directory_absolute(cargo_home)
    try:
        current_class = "generated"
        for component in ("registry", "cache", leaf):
            directory_before = require_directory_fd(
                directory_fd,
                object_class=current_class,
            )
            require_owner_only_mode(directory_before)
            listed = os.stat(component, dir_fd=directory_fd, follow_symlinks=False)
            if not stat.S_ISDIR(listed.st_mode):
                fail("Cargo archive cache path contains a non-directory")
            child_class = (
                "acquisition-registry" if component == "registry" else "generated"
            )
            child_fd = open_directory_at(
                directory_fd,
                component,
                object_class=child_class,
            )
            try:
                opened = require_directory_fd(child_fd, object_class=child_class)
                require_owner_only_mode(opened)
                if not stable_identity(listed, opened):
                    fail("Cargo archive cache directory changed while opening")
                if child_class == "acquisition-registry":
                    require_acquisition_registry_marker(child_fd)
                directory_after = require_directory_fd(
                    directory_fd,
                    object_class=current_class,
                )
                require_owner_only_mode(directory_after)
                if not stable_identity(directory_before, directory_after):
                    fail("Cargo archive cache parent changed during traversal")
            except BaseException:
                os.close(child_fd)
                raise
            os.close(directory_fd)
            directory_fd = child_fd
            current_class = child_class
        return directory_fd
    except BaseException:
        os.close(directory_fd)
        raise


def acquisition_stage_mode(
    lockfile: str,
    archive_dir: str,
    archive_staging: str,
    manifest_authority: str,
) -> None:
    seed_fd = open_directory_absolute(os.path.dirname(lockfile))
    try:
        seed_info = require_directory_fd(
            seed_fd,
            mode=0o500,
            object_class="generated",
        )
        if seed_info.st_mtime_ns != 0:
            fail("acquisition seed payload is not sealed at epoch zero")
        lock_fd, lock_before = open_regular_at(
            seed_fd,
            os.path.basename(lockfile),
            maximum=MANIFEST_BYTES_MAX * 4,
            object_class="generated",
        )
        try:
            if stat.S_IMODE(lock_before.st_mode) != 0o400 or lock_before.st_mtime_ns != 0:
                fail("acquisition seed lockfile is not sealed at epoch zero")
            lock_bytes, lock_after = read_open_file(
                lock_fd,
                MANIFEST_BYTES_MAX * 4,
                object_class="generated",
            )
            if not stable_identity(lock_before, lock_after):
                fail("acquisition seed lockfile changed while reading")
        finally:
            os.close(lock_fd)
    finally:
        os.close(seed_fd)
    if (
        len(lock_bytes) != 108_142
        or hashlib.sha256(lock_bytes).hexdigest()
        != "40a484a61cd15e2555e0a18bd532a8229d78ab4144945b2fe56984c271e7f19b"
    ):
        fail("acquisition seed lockfile is not the exact pre-repair authority")
    packages = parse_lock_bytes(lock_bytes)
    owner = os.path.dirname(archive_staging)
    archive_fd = open_acquisition_archive_directory(owner, archive_dir)
    staging_fd = mkdir_absolute_new(archive_staging)
    manifest_root_fd = mkdir_absolute_new(manifest_authority)
    try:
        expected_names = sorted((package.basename for package in packages), key=raw_key)
        if sorted_entries(archive_fd) != expected_names:
            fail("fresh Cargo archive cache does not exactly match the whole-lock set")
        rows: list[ArchiveRow] = []
        total = 0
        destination_inodes: set[tuple[int, int]] = set()
        source_inodes: set[tuple[int, int]] = set()
        archive_dir_before = require_directory_fd(
            archive_fd,
            object_class="generated",
        )
        for package in packages:
            source_fd, source_before = open_regular_at(
                archive_fd,
                package.basename,
                maximum=ARCHIVE_BYTES_MAX,
                object_class="generated",
            )
            try:
                require_owner_only_mode(source_before)
                source_inode = (source_before.st_dev, source_before.st_ino)
                if source_inode in source_inodes:
                    fail("fresh Cargo archive inputs share an inode")
                source_inodes.add(source_inode)
                size, digest, destination_inode = copy_open_to_new(
                    source_fd,
                    source_before,
                    staging_fd,
                    package.basename,
                    maximum=ARCHIVE_BYTES_MAX,
                    source_object_class="generated",
                )
            finally:
                os.close(source_fd)
            if digest != package.checksum:
                fail("fresh Cargo archive checksum differs from Cargo.lock")
            total = checked_add(total, size, ARCHIVES_BYTES_MAX)
            if destination_inode in destination_inodes or destination_inode in source_inodes:
                fail("staged archives share source or peer inode identity")
            destination_inodes.add(destination_inode)
            rows.append(
                ArchiveRow(
                    package.name,
                    package.version,
                    package.source,
                    package.basename,
                    size,
                    package.checksum,
                )
            )
        archive_dir_after = require_directory_fd(
            archive_fd,
            object_class="generated",
        )
        if (
            not stable_identity(archive_dir_before, archive_dir_after)
            or sorted_entries(archive_fd) != expected_names
        ):
            fail("fresh Cargo archive directory changed during staging")
        if sorted_entries(staging_fd) != expected_names:
            fail("staged archive output set is not exact")
        manifest = encode_manifest(rows)
        create_bytes_at(manifest_root_fd, "archive-manifest-v1.tsv", manifest)
        if sorted_entries(manifest_root_fd) != ["archive-manifest-v1.tsv"]:
            fail("manifest authority output set is not exact")
        manifest_fd, created = open_regular_at(
            manifest_root_fd,
            "archive-manifest-v1.tsv",
            maximum=MANIFEST_BYTES_MAX,
            object_class="generated",
        )
        try:
            size, digest = hash_open_file(
                manifest_fd,
                created,
                object_class="generated",
            )
            if size != len(manifest) or digest != hashlib.sha256(manifest).hexdigest():
                fail("reopened manifest authority bytes changed")
            before = require_regular_fd(
                manifest_fd,
                object_class="generated",
            )
            os.fchmod(manifest_fd, 0o400)
            os.utime(manifest_fd, ns=(0, 0))
            os.fsync(manifest_fd)
            after = require_regular_fd(
                manifest_fd,
                object_class="generated",
            )
            if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
                fail("manifest authority seal changed inode identity")
            if stat.S_IMODE(after.st_mode) != 0o400 or after.st_mtime_ns != 0:
                fail("manifest authority seal state is not exact")
        finally:
            os.close(manifest_fd)
        seal_archive_directory(staging_fd, None)
        seal_directory_fd(manifest_root_fd, 0o500)
    finally:
        os.close(archive_fd)
        os.close(staging_fd)
        os.close(manifest_root_fd)


def archive_seal_mode(archive_dir: str, manifest_path: str, destination: str) -> None:
    archive_fd = open_directory_absolute(archive_dir)
    manifest_fd, manifest_before = open_regular_absolute(
        manifest_path,
        maximum=MANIFEST_BYTES_MAX,
        object_class="generated",
    )
    destination_fd = mkdir_absolute_new(destination)
    try:
        manifest_parent_fd = open_directory_absolute(os.path.dirname(manifest_path))
        try:
            parent_info = require_directory_fd(
                manifest_parent_fd,
                mode=0o500,
                object_class="generated",
            )
            if parent_info.st_mtime_ns != 0:
                fail("manifest authority directory is not sealed at epoch zero")
        finally:
            os.close(manifest_parent_fd)
        if stat.S_IMODE(manifest_before.st_mode) != 0o400 or manifest_before.st_mtime_ns != 0:
            fail("manifest authority file is not sealed at epoch zero")
        manifest_data, manifest_read = read_open_file(
            manifest_fd,
            MANIFEST_BYTES_MAX,
            object_class="generated",
        )
        if not stable_identity(manifest_before, manifest_read):
            fail("manifest authority changed before archive sealing")
        rows = parse_manifest_bytes(manifest_data)
        expected_names = sorted((row.basename for row in rows), key=raw_key)
        if sorted_entries(archive_fd) != expected_names:
            fail("archive staging entries do not match the final manifest")
        total = 0
        source_inodes: set[tuple[int, int]] = {
            (manifest_before.st_dev, manifest_before.st_ino)
        }
        destination_inodes: set[tuple[int, int]] = set()
        directory_before = require_directory_fd(
            archive_fd,
            mode=0o500,
            object_class="generated",
        )
        if directory_before.st_mtime_ns != 0:
            fail("archive staging directory mtime is not epoch zero")
        for row in rows:
            source_fd, source_before = open_regular_at(
                archive_fd,
                row.basename,
                maximum=ARCHIVE_BYTES_MAX,
                object_class="generated",
            )
            try:
                if stat.S_IMODE(source_before.st_mode) != 0o400 or source_before.st_mtime_ns != 0:
                    fail("staged archive is not sealed mode 0400")
                source_inode = (source_before.st_dev, source_before.st_ino)
                if source_inode in source_inodes:
                    fail("staged archives share an inode")
                source_inodes.add(source_inode)
                size, digest, destination_inode = copy_open_to_new(
                    source_fd,
                    source_before,
                    destination_fd,
                    row.basename,
                    maximum=ARCHIVE_BYTES_MAX,
                    source_object_class="generated",
                )
            finally:
                os.close(source_fd)
            if size != row.size or digest != row.checksum:
                fail("staged archive bytes differ from the final manifest")
            total = checked_add(total, size, ARCHIVES_BYTES_MAX)
            if destination_inode in source_inodes or destination_inode in destination_inodes:
                fail("sealed archive destination shares an inode")
            destination_inodes.add(destination_inode)
        manifest_size, manifest_digest, manifest_inode = copy_open_to_new(
            manifest_fd,
            manifest_before,
            destination_fd,
            "archive-manifest-v1.tsv",
            maximum=MANIFEST_BYTES_MAX,
            source_object_class="generated",
        )
        if (
            manifest_size != len(manifest_data)
            or manifest_digest != hashlib.sha256(manifest_data).hexdigest()
        ):
            fail("sealed bundle manifest copy differs from authority")
        if manifest_inode in source_inodes or manifest_inode in destination_inodes:
            fail("sealed bundle manifest shares an inode")
        expected_destination = sorted(expected_names + ["archive-manifest-v1.tsv"], key=raw_key)
        if sorted_entries(destination_fd) != expected_destination:
            fail("sealed bundle output set is not exact")
        directory_after = require_directory_fd(
            archive_fd,
            mode=0o500,
            object_class="generated",
        )
        if (
            not stable_identity(directory_before, directory_after)
            or sorted_entries(archive_fd) != expected_names
        ):
            fail("archive staging changed while sealing bundle")
        manifest_after = require_regular_fd(
            manifest_fd,
            maximum=MANIFEST_BYTES_MAX,
            object_class="generated",
        )
        if not stable_identity(manifest_before, manifest_after):
            fail("manifest authority changed while sealing bundle")
        seal_archive_directory(destination_fd, "archive-manifest-v1.tsv")
    finally:
        os.close(archive_fd)
        os.close(manifest_fd)
        os.close(destination_fd)


@dataclass
class ExtractionBudget:
    compressed: int = 0
    members: int = 0
    expanded: int = 0
    realized: int = 0

    def add_archive(self, size: int) -> None:
        self.compressed = checked_add(self.compressed, size, ARCHIVES_BYTES_MAX)

    def add_members(self, count: int) -> None:
        if count > ARCHIVE_MEMBERS_MAX:
            fail("one archive exceeds the member-count bound")
        self.members = checked_add(self.members, count, ARCHIVES_MEMBERS_MAX)

    def add_expanded_archive(self, size: int) -> None:
        if size > ARCHIVE_EXPANDED_MAX:
            fail("one archive exceeds the expanded-byte bound")
        self.expanded = checked_add(self.expanded, size, ARCHIVES_EXPANDED_MAX)

    def add_realized(self) -> None:
        self.realized = checked_add(self.realized, 1, REALIZED_ENTRIES_MAX)

    def require_realized_capacity(self, count: int) -> None:
        checked_add(self.realized, count, REALIZED_ENTRIES_MAX)


@dataclass(frozen=True)
class PlannedMember:
    member: tarfile.TarInfo
    path: str
    parts: tuple[str, ...]
    executable: bool


def validate_members(
    members: Sequence[tarfile.TarInfo],
    row: ArchiveRow,
    budget: ExtractionBudget,
) -> tuple[PlannedMember, ...]:
    budget.add_members(len(members))
    exact: set[str] = set()
    folded: dict[str, str] = {}
    normalized: dict[str, str] = {}
    planned: list[PlannedMember] = []
    expanded = 0
    top_levels: set[str] = set()
    for member in members:
        if not (member.isdir() or member.isreg()):
            fail("archive contains a link, sparse, or special member")
        if member.mode & 0o7000:
            fail("archive member carries set-id or sticky bits")
        if getattr(member, "sparse", None) or any(
            key.startswith("GNU.sparse") for key in member.pax_headers
        ):
            fail("archive contains a sparse member")
        path = member.name[:-1] if member.isdir() and member.name.endswith("/") else member.name
        parts = validate_relative(path)
        top_levels.add(parts[0])
        if parts[0] != row.crate_root:
            fail("archive member is outside the exact name-version crate root")
        if ".cargo-checksum.json" in parts[1:]:
            fail("archive contains a reserved Cargo checksum member")
        casefolded = path.casefold()
        nfc = unicodedata.normalize("NFC", path)
        if path in exact or (casefolded in folded and folded[casefolded] != path):
            fail("archive contains an exact or default-case-folded path collision")
        if nfc in normalized and normalized[nfc] != path:
            fail("archive contains an NFC-normalized path collision")
        exact.add(path)
        folded[casefolded] = path
        normalized[nfc] = path
        if member.isreg():
            if len(parts) < 2:
                fail("crate root itself cannot be a regular file")
            if member.size < 0 or member.size > FILE_BYTES_MAX:
                fail("archive regular member exceeds its byte bound")
            expanded = checked_add(expanded, member.size, ARCHIVE_EXPANDED_MAX)
        elif member.size not in (0,):
            fail("archive directory member has a nonzero payload")
        planned.append(PlannedMember(member, path, parts, bool(member.mode & 0o111)))
    regular_paths = {item.path for item in planned if item.member.isreg()}
    for item in planned:
        for end in range(1, len(item.parts)):
            if "/".join(item.parts[:end]) in regular_paths:
                fail("archive regular member is the parent of another member")
    if top_levels != {row.crate_root}:
        fail("archive does not contain exactly its one named top-level directory")
    budget.add_expanded_archive(expanded)
    return tuple(planned)


def ensure_directory_path(
    root_fd: int,
    parts: Sequence[str],
    realized: dict[str, str],
    budget: ExtractionBudget,
) -> int:
    current = os.dup(root_fd)
    prefix: list[str] = []
    try:
        for part in parts:
            prefix.append(part)
            relative = "/".join(prefix)
            kind = realized.get(relative)
            if kind is None:
                budget.add_realized()
                child = mkdir_new_at(current, part)
                realized[relative] = "directory"
            elif kind == "directory":
                child = open_directory_at(
                    current,
                    part,
                    object_class="generated",
                )
            else:
                fail("archive parent path collides with a regular file")
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def create_member_file(
    root_fd: int,
    planned: PlannedMember,
    source: BinaryIO,
    realized: dict[str, str],
    budget: ExtractionBudget,
) -> str:
    parent = ensure_directory_path(root_fd, planned.parts[:-1], realized, budget)
    try:
        relative = planned.path
        if relative in realized:
            fail("archive regular member collides with an earlier realized path")
        budget.add_realized()
        destination = create_regular_at(parent, planned.parts[-1])
        digest = hashlib.sha256()
        remaining = planned.member.size
        try:
            while remaining:
                block = source.read(min(READ_CHUNK, remaining))
                if not block:
                    fail("archive regular member ended before its declared size")
                remaining -= len(block)
                digest.update(block)
                write_all(destination, block)
            if source.read(1):
                fail("archive regular member exceeds its declared size")
            os.fsync(destination)
            observed = require_regular_fd(
                destination,
                maximum=FILE_BYTES_MAX,
                object_class="generated",
            )
            if observed.st_size != planned.member.size:
                fail("extracted member size differs from its header")
        finally:
            os.close(destination)
        realized[relative] = "file"
        return digest.hexdigest()
    finally:
        os.close(parent)


def materialize_directory_member(
    root_fd: int,
    planned: PlannedMember,
    realized: dict[str, str],
    explicit: set[str],
    budget: ExtractionBudget,
) -> None:
    if planned.path in explicit:
        fail("archive contains a duplicate explicit directory member")
    explicit.add(planned.path)
    fd = ensure_directory_path(root_fd, planned.parts, realized, budget)
    os.close(fd)


def canonical_checksum_bytes(files: Mapping[str, str], package: str) -> bytes:
    require_sha256(package)
    ordered: dict[str, str] = {}
    for path in sorted(files, key=raw_key):
        validate_relative(path)
        ordered[path] = require_sha256(files[path])
    value = {"files": ordered, "package": package}
    text = json.dumps(value, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
    encoded = text.encode("utf-8", "strict") + b"\n"
    if len(encoded) > FILE_BYTES_MAX:
        fail("generated Cargo checksum file exceeds the regular-file bound")
    if b"\\u" in encoded or not encoded.endswith(b"\n"):
        fail("Cargo checksum JSON is not the canonical direct-UTF-8 encoding")
    return encoded


@dataclass(frozen=True)
class PatchHunk:
    old_start: int
    old_count: int
    new_start: int
    new_count: int
    lines: tuple[bytes, ...]


@dataclass(frozen=True)
class PatchFile:
    path: str
    hunks: tuple[PatchHunk, ...]


HUNK_RE = re.compile(rb"@@ -([1-9][0-9]*)(?:,([0-9]+))? \+([1-9][0-9]*)(?:,([0-9]+))? @@(?: .*)?\n")


def parse_patch(data: bytes) -> tuple[PatchFile, ...]:
    if (
        len(data) != PATCH_BYTES
        or data.count(b"\n") != PATCH_LINES
        or hashlib.sha256(data).hexdigest() != PATCH_SHA256
        or not data.endswith(b"\n")
    ):
        fail("Rocks patch byte identity is not exact")
    lines = data.splitlines(keepends=True)
    index = 0
    results: list[PatchFile] = []
    header_count = 0
    hunk_count = 0
    while index < len(lines):
        if not lines[index].startswith(b"--- a/") or index + 1 >= len(lines):
            fail("Rocks patch file header is malformed")
        old_header = lines[index]
        new_header = lines[index + 1]
        if not new_header.startswith(b"+++ b/"):
            fail("Rocks patch paired file header is malformed")
        try:
            old_path = old_header[6:-1].decode("utf-8", "strict")
            new_path = new_header[6:-1].decode("utf-8", "strict")
        except UnicodeError as exc:
            raise ProvisionError("Rocks patch path is not UTF-8") from exc
        if old_path != new_path or old_path not in ROCKS_FILES:
            fail("Rocks patch changes an unauthorized path")
        validate_relative(old_path)
        header_count += 2
        index += 2
        hunks: list[PatchHunk] = []
        while index < len(lines) and lines[index].startswith(b"@@ "):
            match = HUNK_RE.fullmatch(lines[index])
            if match is None:
                fail("Rocks patch hunk header is malformed")
            old_start = int(match.group(1))
            old_count = int(match.group(2) or b"1")
            new_start = int(match.group(3))
            new_count = int(match.group(4) or b"1")
            index += 1
            body: list[bytes] = []
            observed_old = 0
            observed_new = 0
            while index < len(lines) and not lines[index].startswith((b"@@ ", b"--- a/")):
                line = lines[index]
                if not line.endswith(b"\n") or line[:1] not in (b" ", b"-", b"+"):
                    fail("Rocks patch contains a noncanonical hunk line")
                if line[:1] != b"+":
                    observed_old += 1
                if line[:1] != b"-":
                    observed_new += 1
                body.append(line)
                index += 1
            if observed_old != old_count or observed_new != new_count:
                fail("Rocks patch hunk counts do not match its body")
            hunks.append(PatchHunk(old_start, old_count, new_start, new_count, tuple(body)))
            hunk_count += 1
        if not hunks:
            fail("Rocks patch file has no hunks")
        results.append(PatchFile(old_path, tuple(hunks)))
    if (
        header_count != 4
        or hunk_count != 4
        or len(results) != 2
        or {item.path for item in results} != set(ROCKS_FILES)
        or len({item.path for item in results}) != 2
    ):
        fail("Rocks patch does not have the exact two-file four-hunk shape")
    return tuple(results)


def apply_hunks(original: bytes, patch_file: PatchFile) -> bytes:
    source = original.splitlines(keepends=True)
    output: list[bytes] = []
    source_index = 0
    for hunk in patch_file.hunks:
        declared_source = hunk.old_start - 1
        declared_output = hunk.new_start - 1
        if declared_source < source_index:
            fail("Rocks patch has an overlapping or backward hunk")
        output.extend(source[source_index:declared_source])
        source_index = declared_source
        if len(output) != declared_output:
            fail("Rocks patch requires an offset from its declared new line")
        for line in hunk.lines:
            prefix, content = line[:1], line[1:]
            if prefix in (b" ", b"-"):
                if source_index >= len(source) or source[source_index] != content:
                    fail("Rocks patch context is not exact at its declared line")
                source_index += 1
            if prefix in (b" ", b"+"):
                output.append(content)
    output.extend(source[source_index:])
    return b"".join(output)


def load_patch(path: str) -> tuple[PatchFile, ...]:
    data = read_regular_absolute(
        path,
        PATCH_BYTES,
        object_class="repository-fresh",
    )
    return parse_patch(data)


def register_global_paths(
    planned: Sequence[PlannedMember],
    exact: set[str],
    folded: dict[str, str],
    normalized: dict[str, str],
) -> None:
    for item in planned:
        for end in range(1, len(item.parts) + 1):
            path = "/".join(item.parts[:end])
            casefolded = path.casefold()
            nfc = unicodedata.normalize("NFC", path)
            if path not in exact:
                if casefolded in folded and folded[casefolded] != path:
                    fail("directory source has a cross-archive case-fold collision")
                if nfc in normalized and normalized[nfc] != path:
                    fail("directory source has a cross-archive NFC collision")
                exact.add(path)
                folded[casefolded] = path
                normalized[nfc] = path


def extract_archive(
    archive_fd: int,
    archive_before: os.stat_result,
    row: ArchiveRow,
    destination_fd: int,
    patch_files: Sequence[PatchFile],
    budget: ExtractionBudget,
    realized: dict[str, str],
    executable_paths: dict[str, bool],
    expected_files: dict[str, tuple[int, str]],
    global_exact: set[str],
    global_folded: dict[str, str],
    global_normalized: dict[str, str],
) -> None:
    size, digest = hash_open_file(
        archive_fd,
        archive_before,
        object_class="generated",
    )
    if size != row.size or digest != row.checksum:
        fail("archive descriptor bytes differ from the manifest")
    budget.add_archive(size)
    if row.name == ROCKS_NAME and row.version == ROCKS_VERSION and digest != ROCKS_ARCHIVE_SHA256:
        fail("Rocks archive is not the exact accepted upstream archive")
    os.lseek(archive_fd, 0, os.SEEK_SET)
    raw = os.fdopen(archive_fd, "rb", closefd=False)
    try:
        try:
            archive = tarfile.open(
                fileobj=raw,
                mode="r:gz",
                encoding="utf-8",
                errors="strict",
            )
        except (tarfile.TarError, UnicodeError, OSError) as exc:
            raise ProvisionError("crate archive cannot be opened as strict gzip tar") from exc
        try:
            try:
                members: list[tarfile.TarInfo] = []
                for member in archive:
                    if len(members) >= ARCHIVE_MEMBERS_MAX:
                        fail("one archive exceeds the member-count bound")
                    members.append(member)
            except (tarfile.TarError, UnicodeError, OSError) as exc:
                raise ProvisionError("crate archive member table is invalid") from exc
            planned = validate_members(members, row, budget)
            if row.crate_root in global_exact:
                fail("crate root collides with an earlier archive")
            register_global_paths(planned, global_exact, global_folded, global_normalized)
            checksum_path = row.crate_root + "/.cargo-checksum.json"
            checksum_folded = checksum_path.casefold()
            checksum_normalized = unicodedata.normalize("NFC", checksum_path)
            if checksum_path in global_exact:
                fail("generated Cargo checksum path already exists")
            if (
                checksum_folded in global_folded
                and global_folded[checksum_folded] != checksum_path
            ):
                fail("generated Cargo checksum has a case-fold path collision")
            if (
                checksum_normalized in global_normalized
                and global_normalized[checksum_normalized] != checksum_path
            ):
                fail("generated Cargo checksum has an NFC path collision")
            global_exact.add(checksum_path)
            global_folded[checksum_folded] = checksum_path
            global_normalized[checksum_normalized] = checksum_path
            planned_realized = {checksum_path}
            for item in planned:
                for end in range(1, len(item.parts) + 1):
                    planned_realized.add("/".join(item.parts[:end]))
            new_realized = planned_realized.difference(realized)
            budget.require_realized_capacity(len(new_realized))
            realized_before = len(realized)
            explicit_directories: set[str] = set()
            file_hashes: dict[str, str] = {}
            for item in planned:
                if item.member.isdir():
                    materialize_directory_member(
                        destination_fd,
                        item,
                        realized,
                        explicit_directories,
                        budget,
                    )
                    continue
                if len(item.parts) < 2:
                    fail("crate root itself cannot be a regular file")
                try:
                    source = archive.extractfile(item.member)
                except (tarfile.TarError, OSError) as exc:
                    raise ProvisionError("crate regular member cannot be opened") from exc
                if source is None:
                    fail("crate regular member has no byte stream")
                try:
                    member_digest = create_member_file(
                        destination_fd,
                        item,
                        source,
                        realized,
                        budget,
                    )
                finally:
                    source.close()
                relative_in_crate = "/".join(item.parts[1:])
                file_hashes[relative_in_crate] = member_digest
                executable_paths[item.path] = item.executable
                expected_files[item.path] = (item.member.size, member_digest)
        finally:
            archive.close()
    finally:
        raw.close()

    is_rocks = row.name == ROCKS_NAME and row.version == ROCKS_VERSION
    if is_rocks:
        if len(patch_files) != 2:
            fail("Rocks patch authority is absent")
        for patch_file in patch_files:
            upstream_hash, patched_hash = ROCKS_FILES[patch_file.path]
            full_path = row.crate_root + "/" + patch_file.path
            original = read_bytes_in_tree(
                destination_fd,
                full_path,
                FILE_BYTES_MAX,
                object_class="generated",
            )
            if hashlib.sha256(original).hexdigest() != upstream_hash:
                fail("Rocks source preimage hash differs from accepted upstream")
            patched = apply_hunks(original, patch_file)
            if hashlib.sha256(patched).hexdigest() != patched_hash:
                fail("Rocks patched source hash differs from accepted output")
            replace_relative_file(destination_fd, full_path, original, patched)
            file_hashes[patch_file.path] = patched_hash
            expected_files[full_path] = (len(patched), patched_hash)

    checksum = canonical_checksum_bytes(file_hashes, row.checksum)
    if checksum_path in realized:
        fail("generated Cargo checksum path already exists")
    budget.add_realized()
    crate_fd = open_directory_at(
        destination_fd,
        row.crate_root,
        object_class="generated",
    )
    try:
        create_bytes_at(crate_fd, ".cargo-checksum.json", checksum)
    finally:
        os.close(crate_fd)
    realized[checksum_path] = "file"
    executable_paths[checksum_path] = False
    expected_files[checksum_path] = (
        len(checksum),
        hashlib.sha256(checksum).hexdigest(),
    )
    if len(realized) - realized_before != len(new_realized):
        fail("archive realized-entry count differs from its prevalidated plan")

    archive_after = require_regular_fd(
        archive_fd,
        maximum=ARCHIVE_BYTES_MAX,
        object_class="generated",
    )
    if not stable_identity(archive_before, archive_after):
        fail("archive descriptor changed during validation and extraction")


def source_mode(
    archive_dir: str,
    manifest_path: str,
    patch_path: str,
    destination: str,
) -> None:
    archive_fd = open_directory_absolute(archive_dir)
    manifest_fd, manifest_before = open_regular_absolute(
        manifest_path,
        maximum=MANIFEST_BYTES_MAX,
        object_class="generated",
    )
    patch = load_patch(patch_path)
    destination_fd = mkdir_absolute_new(destination)
    try:
        manifest_parent_fd = open_directory_absolute(os.path.dirname(manifest_path))
        try:
            manifest_parent = require_directory_fd(
                manifest_parent_fd,
                mode=0o500,
                object_class="generated",
            )
            if manifest_parent.st_mtime_ns != 0:
                fail("archive manifest parent is not sealed at epoch zero")
        finally:
            os.close(manifest_parent_fd)
        archive_root_info = require_directory_fd(
            archive_fd,
            object_class="generated",
        )
        if stat.S_IMODE(archive_root_info.st_mode) != 0o500 or archive_root_info.st_mtime_ns != 0:
            fail("archive source directory is not sealed mode 0500")
        if stat.S_IMODE(manifest_before.st_mode) != 0o400 or manifest_before.st_mtime_ns != 0:
            fail("archive manifest input is not sealed mode 0400")
        manifest_data, manifest_read = read_open_file(
            manifest_fd,
            MANIFEST_BYTES_MAX,
            object_class="generated",
        )
        if not stable_identity(manifest_before, manifest_read):
            fail("archive manifest changed before source extraction")
        rows = parse_manifest_bytes(manifest_data)
        if sum(row.name == ROCKS_NAME and row.version == ROCKS_VERSION for row in rows) != 1:
            fail("manifest does not contain exactly one accepted Rocks package")
        expected_entries = {row.basename for row in rows}
        if os.path.dirname(manifest_path) == archive_dir:
            if os.path.basename(manifest_path) != "archive-manifest-v1.tsv":
                fail("bundle manifest basename is not exact")
            expected_entries.add("archive-manifest-v1.tsv")
        if set(sorted_entries(archive_fd)) != expected_entries:
            fail("archive input directory does not exactly match the manifest")
        directory_before = require_directory_fd(
            archive_fd,
            mode=0o500,
            object_class="generated",
        )
        budget = ExtractionBudget()
        realized: dict[str, str] = {}
        executable_paths: dict[str, bool] = {}
        expected_files: dict[str, tuple[int, str]] = {}
        global_exact: set[str] = set()
        global_folded: dict[str, str] = {}
        global_normalized: dict[str, str] = {}
        source_inodes: set[tuple[int, int]] = {
            (manifest_before.st_dev, manifest_before.st_ino)
        }
        for row in rows:
            member_fd, member_before = open_regular_at(
                archive_fd,
                row.basename,
                maximum=ARCHIVE_BYTES_MAX,
                object_class="generated",
            )
            try:
                if stat.S_IMODE(member_before.st_mode) != 0o400 or member_before.st_mtime_ns != 0:
                    fail("archive input file is not sealed mode 0400")
                inode = (member_before.st_dev, member_before.st_ino)
                if inode in source_inodes:
                    fail("archive input set contains shared file inodes")
                source_inodes.add(inode)
                extract_archive(
                    member_fd,
                    member_before,
                    row,
                    destination_fd,
                    patch,
                    budget,
                    realized,
                    executable_paths,
                    expected_files,
                    global_exact,
                    global_folded,
                    global_normalized,
                )
            finally:
                os.close(member_fd)
        if budget.realized != len(realized):
            fail("realized source-entry accounting is inconsistent")
        directory_after = require_directory_fd(
            archive_fd,
            mode=0o500,
            object_class="generated",
        )
        if (
            not stable_identity(directory_before, directory_after)
            or set(sorted_entries(archive_fd)) != expected_entries
        ):
            fail("archive source directory changed during extraction")
        manifest_after = require_regular_fd(
            manifest_fd,
            maximum=MANIFEST_BYTES_MAX,
            object_class="generated",
        )
        if not stable_identity(manifest_before, manifest_after):
            fail("archive manifest changed during source extraction")
        physical_directories, physical_files = inventory_tree(
            destination_fd,
            object_class="generated",
        )
        expected_directories = {path for path, kind in realized.items() if kind == "directory"}
        expected_file_paths = {path for path, kind in realized.items() if kind == "file"}
        if (
            physical_directories != expected_directories
            or set(physical_files) != expected_file_paths
            or set(expected_files) != expected_file_paths
        ):
            fail("materialized directory-source entries differ from validated archive entries")
        for path, (expected_size, expected_digest) in expected_files.items():
            observed = physical_files[path]
            if observed.size != expected_size or observed.sha256 != expected_digest:
                fail("materialized directory-source file bytes differ from the validated stream")
        seal_tree(destination_fd, executable_paths)
        seal_directory_fd(destination_fd, 0o555)
    finally:
        os.close(archive_fd)
        os.close(manifest_fd)
        os.close(destination_fd)


GRAMMAR: Mapping[str, tuple[tuple[str, str], ...]] = {
    "acquisition-stage": (
        ("--lockfile", "lockfile"),
        ("--archive-dir", "archive_dir"),
        ("--archive-staging", "archive_staging"),
        ("--manifest-authority", "manifest_authority"),
    ),
    "lock-payload": (
        ("--payload-source", "payload_source"),
        ("--destination-root", "destination_root"),
        ("--manifest", "manifest"),
        ("--root-config", "root_config"),
    ),
    "lock-source": (
        ("--archive-dir", "archive_dir"),
        ("--manifest", "manifest"),
        ("--patch", "patch"),
        ("--destination", "destination"),
    ),
    "archive-seal": (
        ("--archive-dir", "archive_dir"),
        ("--manifest", "manifest"),
        ("--destination", "destination"),
    ),
    "qualification-payload": (
        ("--payload-source", "payload_source"),
        ("--destination-root", "destination_root"),
        ("--root-config", "root_config"),
    ),
    "qualification-source": (
        ("--archive-dir", "archive_dir"),
        ("--manifest", "manifest"),
        ("--patch", "patch"),
        ("--destination", "destination"),
    ),
}


def parse_argv(argv: Sequence[str]) -> tuple[str, dict[str, str]]:
    if not argv or argv[0] not in GRAMMAR:
        fail("provisioner mode is not one of the exact six closed modes")
    mode = argv[0]
    grammar = GRAMMAR[mode]
    if len(argv) != 1 + 2 * len(grammar):
        fail("provisioner argv length does not match its closed mode grammar")
    values: dict[str, str] = {}
    offset = 1
    for flag, key in grammar:
        if argv[offset] != flag:
            fail("provisioner flags are absent, duplicated, unknown, or reordered")
        value = argv[offset + 1]
        raw_key(value)
        if not value.startswith("/"):
            fail("every provisioner path argument must be absolute")
        values[key] = value
        offset += 2
    return mode, values


def require_acquisition_root(path: str) -> None:
    if re.fullmatch(r"/private/tmp/engram-c2b2a-acquisition-[A-Za-z0-9._-]+", path) is None:
        fail("acquisition root does not match the frozen grammar")


def require_repository_support(path: str, relative: str) -> None:
    if path != REPOSITORY_PAYLOAD + "/" + relative:
        fail("repository support authority path is not the exact payload-relative path")


def validate_authority_paths(mode: str, values: Mapping[str, str], owner: str) -> None:
    if mode in ("acquisition-stage", "archive-seal"):
        require_acquisition_root(owner)
    elif mode in ("lock-payload", "lock-source"):
        acquisition = os.path.dirname(owner)
        require_acquisition_root(acquisition)
        if owner != acquisition + "/lock-root":
            fail("lock-root path is not exact")

    if mode == "acquisition-stage":
        validate_absolute(values["lockfile"])
        validate_absolute(values["archive_dir"])
        require_absent(values["archive_staging"])
        require_absent(values["manifest_authority"])
        cache_prefix = owner + "/cargo-home/registry/cache/"
        remainder = (
            values["archive_dir"][len(cache_prefix) :]
            if values["archive_dir"].startswith(cache_prefix)
            else ""
        )
        if not remainder or "/" in remainder:
            fail("fresh Cargo cache directory is not its one exact registry leaf")
        require_disjoint(
            values["archive_dir"],
            values["archive_staging"],
            values["manifest_authority"],
        )
    elif mode == "lock-payload":
        validate_absolute(values["payload_source"])
        validate_absolute(values["manifest"])
        validate_absolute(values["root_config"])
        if values["payload_source"] != REPOSITORY_PAYLOAD:
            fail("lock payload source is not the exact repository candidate")
        require_repository_support(values["root_config"], "build-support/root-cargo-config.toml")
        if (
            values["root_config"]
            != values["payload_source"] + "/build-support/root-cargo-config.toml"
        ):
            fail("lock payload root configuration is not from its repository candidate")
        acquisition = os.path.dirname(owner)
        if values["manifest"] != acquisition + "/manifest-authority/archive-manifest-v1.tsv":
            fail("lock payload manifest authority path is not exact")
        require_absent(owner + "/payload")
        require_absent(owner + "/.cargo")
        require_disjoint(values["payload_source"], owner)
    elif mode == "lock-source":
        acquisition = os.path.dirname(owner)
        if values["archive_dir"] != acquisition + "/archive-staging":
            fail("lock source archive staging path is not exact")
        if values["manifest"] != acquisition + "/manifest-authority/archive-manifest-v1.tsv":
            fail("lock source manifest authority path is not exact")
        require_repository_support(
            values["patch"],
            "patches/surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch",
        )
        for key in ("archive_dir", "manifest", "patch"):
            validate_absolute(values[key])
        require_absent(values["destination"])
        require_disjoint(values["archive_dir"], values["destination"])
    elif mode == "archive-seal":
        if values["manifest"] != owner + "/manifest-authority/archive-manifest-v1.tsv":
            fail("archive seal manifest authority path is not exact")
        for key in ("archive_dir", "manifest"):
            validate_absolute(values[key])
        require_absent(values["destination"])
        require_disjoint(values["archive_dir"], values["destination"])
    elif mode == "qualification-payload":
        validate_absolute(values["payload_source"])
        validate_absolute(values["root_config"])
        if values["payload_source"] != REPOSITORY_PAYLOAD:
            fail("qualification payload source is not the exact repository candidate")
        require_repository_support(values["root_config"], "build-support/root-cargo-config.toml")
        if (
            values["root_config"]
            != values["payload_source"] + "/build-support/root-cargo-config.toml"
        ):
            fail("qualification root configuration is not from its repository candidate")
        require_absent(owner + "/payload")
        require_absent(owner + "/.cargo")
        require_disjoint(values["payload_source"], owner)
    elif mode == "qualification-source":
        for key in ("archive_dir", "manifest", "patch"):
            validate_absolute(values[key])
        acquisition = os.path.dirname(values["archive_dir"])
        require_acquisition_root(acquisition)
        if values["archive_dir"] != acquisition + "/sealed-archives":
            fail("qualification archive input is not the exact sealed bundle")
        if values["manifest"] != values["archive_dir"] + "/archive-manifest-v1.tsv":
            fail("qualification manifest is not the manifest inside its sealed bundle")
        require_repository_support(
            values["patch"],
            "patches/surrealdb-librocksdb-sys-0.17.3-rocksdb-getentropy.patch",
        )
        require_absent(values["destination"])
        require_disjoint(values["archive_dir"], values["destination"])
    else:
        fail("unreachable provisioner mode")


def dispatch(mode: str, values: Mapping[str, str]) -> None:
    owner, phase = require_exact_environment(mode, values)
    require_owner_grammar(mode, owner)
    begin_metadata_scope(mode, owner)
    if mode == "acquisition-stage":
        bind_acquisition_registry(owner + "/cargo-home/registry")
    if mode == "qualification-source":
        acquisition = os.path.dirname(values["archive_dir"])
        require_acquisition_root(acquisition)
        add_generated_root(acquisition)
    require_phase_layout(mode, values, owner, phase)
    validate_authority_paths(mode, values, owner)
    if mode == "acquisition-stage":
        acquisition_stage_mode(
            values["lockfile"],
            values["archive_dir"],
            values["archive_staging"],
            values["manifest_authority"],
        )
    elif mode == "lock-payload":
        copy_payload_mode(
            values["payload_source"],
            values["destination_root"],
            values["root_config"],
            values["manifest"],
        )
    elif mode == "lock-source":
        source_mode(
            values["archive_dir"],
            values["manifest"],
            values["patch"],
            values["destination"],
        )
    elif mode == "archive-seal":
        archive_seal_mode(values["archive_dir"], values["manifest"], values["destination"])
    elif mode == "qualification-payload":
        copy_payload_mode(
            values["payload_source"],
            values["destination_root"],
            values["root_config"],
            None,
        )
    elif mode == "qualification-source":
        source_mode(
            values["archive_dir"],
            values["manifest"],
            values["patch"],
            values["destination"],
        )
    else:
        fail("unreachable provisioner mode")


def main(argv: Sequence[str]) -> int:
    try:
        mode, values = parse_argv(argv)
        dispatch(mode, values)
    except ProvisionError as exc:
        sys.stderr.write("provision.py: " + str(exc) + "\n")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
