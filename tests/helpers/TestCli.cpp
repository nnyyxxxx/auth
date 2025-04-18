#include "TestCli.hpp"
#include "../../src/db/SecretStorage.hpp"
#include <iostream>
#include <filesystem>
#include <cstdlib>
#include <string>
#include <vector>

static CTestEnvSetup g_testEnvSetup;

CTestEnvSetup::CTestEnvSetup() {
    const char* envVal = getenv("AUTH_DATABASE_DIR");
    if (envVal)
        m_origDbDir = envVal;

    const char* testDbDir = "/tmp/auth_test_dir";

    if (!std::filesystem::exists(testDbDir))
        std::filesystem::create_directories(testDbDir);

    setenv("AUTH_DATABASE_DIR", testDbDir, 1);
}

CTestEnvSetup::~CTestEnvSetup() {
    if (!m_origDbDir.empty())
        setenv("AUTH_DATABASE_DIR", m_origDbDir.c_str(), 1);
    else
        unsetenv("AUTH_DATABASE_DIR");

    std::filesystem::remove_all("/tmp/auth_test_dir");

    if (CSecretStorage::isAvailable()) {
        CSecretStorage                 secretStorage;

        const std::vector<std::string> testEntryNames = {"TestEntry",     "Entry1",   "Entry2", "TestEntry2", "DeleteTest", "UpdateTest", "Test Entry",
                                                         "Updated Entry", "TestName", "Test",   "Entry 1",    "Entry 2",    "Entry 3"};

        for (const auto& name : testEntryNames) {
            secretStorage.deleteSecretByName(name);
        }
    }
}

CTestAuthCLI::CTestAuthCLI() : CAuthCLI() {
    m_db = std::make_unique<CMockAuthDB>();
}

CMockAuthDB* CTestAuthCLI::getMockDb() {
    return static_cast<CMockAuthDB*>(m_db.get());
}

bool CTestAuthCLI::runCommand(const std::string& command, const std::vector<std::string>& args) {
    std::vector<std::string> fullArgs;

    if (command.empty())
        fullArgs = {"auth"};
    else {
        fullArgs = {"auth", command};
        fullArgs.insert(fullArgs.end(), args.begin(), args.end());
    }

    std::vector<char*> cArgs;
    for (auto& arg : fullArgs) {
        cArgs.push_back(const_cast<char*>(arg.c_str()));
    }

    std::streambuf* oldCoutBuf = std::cout.rdbuf();
    std::streambuf* oldCerrBuf = std::cerr.rdbuf();

    m_stdoutCapture.str("");
    m_stderrCapture.str("");

    std::cout.rdbuf(m_stdoutCapture.rdbuf());
    std::cerr.rdbuf(m_stderrCapture.rdbuf());

    bool result = processCommand(static_cast<int>(cArgs.size()), cArgs.data());

    std::cout.rdbuf(oldCoutBuf);
    std::cerr.rdbuf(oldCerrBuf);

    return result;
}

std::string CTestAuthCLI::getStdout() const {
    return m_stdoutCapture.str();
}

std::string CTestAuthCLI::getStderr() const {
    return m_stderrCapture.str();
}

void CTestAuthCLI::resetCapture() {
    m_stdoutCapture.str("");
    m_stderrCapture.str("");
}