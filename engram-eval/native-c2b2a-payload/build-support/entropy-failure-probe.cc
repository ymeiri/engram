#include <cstddef>
#include <cstring>
#include <string>

#include "rocksdb/env.h"

namespace {

constexpr std::size_t kUuidBytes = 36;
constexpr int kSuccess = 0;
constexpr int kNullOutput = 1;
constexpr int kWrongOutputSize = 2;

}  // namespace

extern "C" int c2b2a_generate_unique_id(unsigned char output[kUuidBytes]) noexcept {
  if (output == nullptr) {
    return kNullOutput;
  }

  const std::string value = ROCKSDB_NAMESPACE::Env::Default()->GenerateUniqueId();
  if (value.size() != kUuidBytes) {
    return kWrongOutputSize;
  }

  std::memcpy(output, value.data(), kUuidBytes);
  return kSuccess;
}
