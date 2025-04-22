#pragma once

#include <string>
#include <vector>
#include <cstdint>
#include <optional>
#include "../db/Db.hpp"

void                      stringToLowerInPlace(std::string& str);
std::string               stringToLower(const std::string& str);
std::vector<uint8_t>      decodeBase32(const std::string& input);
std::string               getHomeDir();
std::optional<SAuthEntry> findEntryByNameOrId(const std::vector<SAuthEntry>& entries, const std::string& nameOrId);
bool                      validateDigits(uint32_t digits, std::string& errorMessage);
bool                      validatePeriod(uint32_t period, std::string& errorMessage);
bool                      isSecretValid(const std::string& secret, std::string& errorMessage);
std::string               getDatabasePath();
std::vector<std::string>  splitString(const std::string& input, const std::string& delimiter);
std::string               truncateWithEllipsis(const std::string& str, size_t maxLength);
