#include "Totp.hpp"
#include "../helpers/MiscFunctions.hpp"
#include <openssl/hmac.h>
#include <openssl/evp.h>
#include <ctime>
#include <iomanip>
#include <sstream>
#include <vector>
#include <cstring>
#include <stdexcept>
#include <cmath>

CTotp::CTotp(const std::string& secret, uint32_t digits, uint32_t period) : m_secret(secret), m_digits(digits), m_period(period) {
    if (m_period == 0)
        m_period = 30;
}

std::string CTotp::generate() const {
    std::vector<uint8_t> key = decodeBase32(m_secret);
    if (key.empty())
        return "Invalid key";

    time_t   currentTime = time(nullptr);
    uint64_t counter     = static_cast<uint64_t>(currentTime) / m_period;

    uint8_t  counterBytes[8] = {0};
    for (int i = 7; i >= 0; i--) {
        counterBytes[i] = counter & 0xFF;
        counter >>= 8;
    }

    uint8_t      hash[EVP_MAX_MD_SIZE];
    unsigned int hashLen = 0;

    HMAC(EVP_sha1(), key.data(), static_cast<int>(key.size()), counterBytes, sizeof(counterBytes), hash, &hashLen);

    int               offset        = hash[hashLen - 1] & 0x0F;
    uint32_t          truncatedHash = ((hash[offset] & 0x7F) << 24) | ((hash[offset + 1] & 0xFF) << 16) | ((hash[offset + 2] & 0xFF) << 8) | (hash[offset + 3] & 0xFF);

    uint32_t          totpValue = truncatedHash % static_cast<uint32_t>(std::pow(10, m_digits));

    std::stringstream ss;
    ss << std::setw(m_digits) << std::setfill('0') << totpValue;
    return ss.str();
}