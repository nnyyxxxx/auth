#pragma once

#include <cstdint>
#include <string>
#include <memory>
#include <optional>

struct SAuthEntry;

class CSecretStorage {
  public:
    CSecretStorage();
    ~CSecretStorage();

    std::string storeSecret(const std::string& name, uint64_t id, const std::string& secret);
    std::string getSecret(const std::string& secretId);
    bool        deleteSecret(const std::string& secretId);
    bool        deleteSecretByName(const std::string& name);
    std::string updateSecret(const std::string& secretId, const std::string& name, uint64_t id, const std::string& newSecret);
    static bool isAvailable();

  private:
    static const char* m_schemaName;
    void               initSchema();
    void*              m_schema;
};