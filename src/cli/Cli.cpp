#include "Cli.hpp"
#include "../helpers/Color.hpp"
#include "../core/Totp.hpp"
#include "../helpers/ImportExport.hpp"
#include "../helpers/MiscFunctions.hpp"
#include <iostream>
#include <filesystem>
#include <pwd.h>
#include <unistd.h>
#include <algorithm>
#include <iomanip>
#include <sstream>

CAuthCLI::CAuthCLI() {
    std::string dbPath = GetDatabasePath();
    if (dbPath.empty())
        return;

    const std::filesystem::path configPath = std::filesystem::path(dbPath).parent_path();
    if (!std::filesystem::exists(configPath))
        std::filesystem::create_directories(configPath);

    m_db = std::make_unique<CFileAuthDB>(dbPath);
    m_db->load();
}

void CAuthCLI::printUsage() {
    std::cout << CColor::BOLD << "Usage: " << CColor::CYAN << "auth" << CColor::RESET << " [command] [options]\n\n";
    std::cout << CColor::BOLD << "Commands:" << CColor::RESET << "\n";
    std::cout << "  " << CColor::GREEN << "add" << CColor::RESET << "      <n> <secret> [digits] [period]                   Add a new TOTP entry\n";
    std::cout << "  " << CColor::GREEN << "list" << CColor::RESET << "                                                      List all entries\n";
    std::cout << "  " << CColor::GREEN << "generate" << CColor::RESET << " <name or #>                                      Generate TOTP code for specific entry\n";
    std::cout << "  " << CColor::GREEN << "remove" << CColor::RESET << "   <name or #>                                      Remove an entry\n";
    std::cout << "  " << CColor::GREEN << "info" << CColor::RESET << "     <name or #>                                      Show details for an entry\n";
    std::cout << "  " << CColor::GREEN << "edit" << CColor::RESET << "     <name or #> [name] [secret] [digits] [period]    Edit an entry\n";
    std::cout << "  " << CColor::GREEN << "import" << CColor::RESET << "   <file> [format]                                  Import entries from file\n";
    std::cout << "  " << CColor::GREEN << "export" << CColor::RESET << "   <file> [format]                                  Export entries to file\n";
    std::cout << "  " << CColor::GREEN << "wipe" << CColor::RESET << "                                                      Wipe database\n";
    std::cout << "  " << CColor::GREEN << "help" << CColor::RESET << "                                                      Show this help message\n";
    std::cout << "\n" << CColor::BOLD << "Options:" << CColor::RESET << "\n";
    std::cout << "  " << CColor::YELLOW << "digits" << CColor::RESET << "   Number of digits in the code                (default: 6)\n";
    std::cout << "  " << CColor::YELLOW << "period" << CColor::RESET << "   Time period in seconds                      (default: 30)\n";
    std::cout << "  " << CColor::YELLOW << "format" << CColor::RESET << "   File format for import/export: toml or json (default: toml)\n";
}

bool CAuthCLI::processCommand(int argc, char* argv[]) {
    if (argc < 2) {
        printUsage();
        return true;
    }

    std::string              command = argv[1];
    std::vector<std::string> args;

    for (int i = 2; i < argc; i++) {
        args.push_back(argv[i]);
    }

    if (command == "add")
        return commandAdd(args);
    else if (command == "remove")
        return commandRemove(args);
    else if (command == "list")
        return commandList();
    else if (command == "generate")
        return commandGenerate(args);
    else if (command == "info")
        return commandInfo(args);
    else if (command == "edit")
        return commandEdit(args);
    else if (command == "import")
        return commandImport(args);
    else if (command == "export")
        return commandExport(args);
    else if (command == "wipe")
        return commandWipe();
    else if (command == "help") {
        printUsage();
        return true;
    } else {
        std::cerr << "Unknown command: " << command << "\n";
        return false;
    }
}

bool CAuthCLI::commandAdd(const std::vector<std::string>& args) {
    if (args.size() < 2) {
        std::cerr << CColor::RED << "Not enough arguments for add command" << CColor::RESET << "\n";
        std::cerr << "Usage: auth add <n> <secret> [digits] [period]\n";
        return false;
    }

    std::string name   = args[0];
    std::string secret = args[1];
    uint32_t    digits = 6;
    uint32_t    period = 30;
    std::string errorMessage;

    if (args.size() >= 3) {
        try {
            digits = std::stoi(args[2]);
            if (!ValidateDigits(digits, errorMessage)) {
                std::cerr << CColor::RED << errorMessage << CColor::RESET << "\n";
                return false;
            }
        } catch (const std::exception& e) {
            std::cerr << CColor::RED << "Invalid digits value" << CColor::RESET << "\n";
            return false;
        }
    }

    if (args.size() >= 4) {
        try {
            period = std::stoi(args[3]);
            if (!ValidatePeriod(period, errorMessage)) {
                std::cerr << CColor::RED << errorMessage << CColor::RESET << "\n";
                return false;
            }
        } catch (const std::exception& e) {
            std::cerr << CColor::RED << "Invalid period value" << CColor::RESET << "\n";
            return false;
        }
    }

    if (!IsSecretValid(secret, errorMessage)) {
        std::cerr << CColor::RED << errorMessage << CColor::RESET << "\n";
        return false;
    }

    SAuthEntry entry;
    entry.name   = name;
    entry.secret = secret;
    entry.digits = digits;
    entry.period = period;

    if (m_db->addEntry(entry)) {
        std::cout << CColor::GREEN << "Added new entry: " << name << CColor::RESET << "\n";
        return true;
    } else {
        std::cerr << CColor::RED << "Failed to add entry" << CColor::RESET << "\n";
        return false;
    }
}

bool CAuthCLI::commandRemove(const std::vector<std::string>& args) {
    if (args.empty()) {
        std::cerr << CColor::RED << "Missing argument for remove command" << CColor::RESET << "\n";
        std::cerr << "Usage: auth remove <name or id>\n";
        return false;
    }

    std::string nameOrId = args[0];
    auto        entries  = m_db->getEntries();

    auto        entryOpt = FindEntryByNameOrId(entries, nameOrId);
    if (!entryOpt) {
        std::cerr << CColor::RED << "Entry not found: " << nameOrId << CColor::RESET << "\n";
        return false;
    }

    const auto& entry = *entryOpt;

    if (m_db->removeEntry(entry.id)) {
        std::cout << CColor::GREEN << "Removed entry: " << entry.name << CColor::RESET << "\n";
        return true;
    }

    std::cerr << CColor::RED << "Failed to remove entry" << CColor::RESET << "\n";
    return false;
}

bool CAuthCLI::commandList() {
    auto entries = m_db->getEntries();

    if (entries.empty()) {
        std::cout << CColor::YELLOW << "No entries found" << CColor::RESET << "\n";
        return true;
    }

    std::ranges::sort(entries, [](const SAuthEntry& a, const SAuthEntry& b) { return a.id < b.id; });

    size_t maxNameLength = 0;
    for (const auto& entry : entries) {
        maxNameLength = std::max(maxNameLength, entry.name.length());
    }

    std::cout << CColor::BOLD << std::left << std::setw(5) << "#" << std::setw(maxNameLength + 2) << "NAME" << "CODE" << "    EXPIRES" << CColor::RESET << "\n";

    time_t now       = time(nullptr);
    size_t rowNumber = 1;

    for (const auto& entry : entries) {
        CTOTP       totp(entry.secret, entry.digits, entry.period);
        std::string code = totp.generate();

        int         periodRemaining = entry.period - (now % entry.period);

        std::cout << CColor::CYAN << std::left << std::setw(5) << rowNumber++ << CColor::RESET << CColor::GREEN << std::setw(maxNameLength + 2) << entry.name << CColor::RESET
                  << CColor::BOLD << CColor::YELLOW << std::setw(8) << code << CColor::RESET << " " << CColor::MAGENTA << periodRemaining << "s" << CColor::RESET << "\n";
    }

    return true;
}

bool CAuthCLI::commandGenerate(const std::vector<std::string>& args) {
    if (args.empty()) {
        std::cerr << CColor::RED << "Missing argument for generate command" << CColor::RESET << "\n";
        std::cerr << "Usage: auth generate <name or id>\n";
        return false;
    }

    std::string nameOrId = args[0];
    auto        entries  = m_db->getEntries();

    auto        entryOpt = FindEntryByNameOrId(entries, nameOrId);
    if (!entryOpt) {
        std::cerr << CColor::RED << "Entry not found: " << nameOrId << CColor::RESET << "\n";
        return false;
    }

    CTOTP       totp(entryOpt->secret, entryOpt->digits, entryOpt->period);
    std::string code = totp.generate();

    std::cout << CColor::YELLOW << code << CColor::RESET << std::endl;
    return true;
}

bool CAuthCLI::commandInfo(const std::vector<std::string>& args) {
    if (args.empty()) {
        std::cerr << CColor::RED << "Missing argument for info command" << CColor::RESET << "\n";
        std::cerr << "Usage: auth info <name or id>\n";
        return false;
    }

    std::string nameOrId = args[0];
    auto        entries  = m_db->getEntries();

    auto        entryOpt = FindEntryByNameOrId(entries, nameOrId);
    if (!entryOpt) {
        std::cerr << CColor::RED << "Entry not found: " << nameOrId << CColor::RESET << "\n";
        return false;
    }

    const auto& entry = *entryOpt;
    std::cout << CColor::BOLD << "Name:   " << CColor::RESET << CColor::GREEN << entry.name << CColor::RESET << "\n";
    std::cout << CColor::BOLD << "ID:     " << CColor::RESET << CColor::CYAN << entry.id << CColor::RESET << "\n";
    std::cout << CColor::BOLD << "Secret: " << CColor::RESET << entry.secret << "\n";
    std::cout << CColor::BOLD << "Digits: " << CColor::RESET << entry.digits << "\n";
    std::cout << CColor::BOLD << "Period: " << CColor::RESET << entry.period << "s\n";

    CTOTP       totp(entry.secret, entry.digits, entry.period);
    std::string code = totp.generate();

    time_t      now             = time(nullptr);
    int         periodRemaining = entry.period - (now % entry.period);

    std::cout << CColor::BOLD << "Code:   " << CColor::RESET << CColor::YELLOW << code << CColor::RESET << " (expires in " << CColor::MAGENTA << periodRemaining << "s"
              << CColor::RESET << ")\n";
    return true;
}

bool CAuthCLI::commandEdit(const std::vector<std::string>& args) {
    if (args.empty()) {
        std::cerr << CColor::RED << "Missing arguments for edit command" << CColor::RESET << "\n";
        std::cerr << "Usage: auth edit <name or id> [name] [secret] [digits] [period]\n";
        return false;
    }

    std::string nameOrId = args[0];
    auto        entries  = m_db->getEntries();

    auto        entryOpt = FindEntryByNameOrId(entries, nameOrId);
    if (!entryOpt) {
        std::cerr << CColor::RED << "Entry not found: " << nameOrId << CColor::RESET << "\n";
        return false;
    }

    SAuthEntry  entryToEdit  = *entryOpt;
    std::string originalName = entryToEdit.name;
    std::string errorMessage;

    if (args.size() > 1 && !args[1].empty())
        entryToEdit.name = args[1];

    if (args.size() > 2 && !args[2].empty()) {
        std::string secret = args[2];

        if (!IsSecretValid(secret, errorMessage)) {
            std::cerr << CColor::RED << errorMessage << CColor::RESET << "\n";
            return false;
        }

        entryToEdit.secret = secret;
    }

    if (args.size() > 3 && !args[3].empty()) {
        try {
            uint32_t digits = std::stoi(args[3]);
            if (!ValidateDigits(digits, errorMessage)) {
                std::cerr << CColor::RED << errorMessage << CColor::RESET << "\n";
                return false;
            }
            entryToEdit.digits = digits;
        } catch (const std::exception& e) {
            std::cerr << CColor::RED << "Invalid digits value" << CColor::RESET << "\n";
            return false;
        }
    }

    if (args.size() > 4 && !args[4].empty()) {
        try {
            uint32_t period = std::stoi(args[4]);
            if (!ValidatePeriod(period, errorMessage)) {
                std::cerr << CColor::RED << errorMessage << CColor::RESET << "\n";
                return false;
            }
            entryToEdit.period = period;
        } catch (const std::exception& e) {
            std::cerr << CColor::RED << "Invalid period value" << CColor::RESET << "\n";
            return false;
        }
    }

    if (m_db->updateEntry(entryToEdit)) {
        std::cout << CColor::GREEN << "Updated entry: " << originalName << CColor::RESET << "\n";
        return true;
    } else {
        std::cerr << CColor::RED << "Failed to update entry" << CColor::RESET << "\n";
        return false;
    }
}

bool CAuthCLI::commandImport(const std::vector<std::string>& args) {
    if (args.empty()) {
        std::cerr << CColor::RED << "Missing argument for import command" << CColor::RESET << "\n";
        std::cerr << "Usage: auth import <file> [format]\n";
        std::cerr << "Supported formats: toml, json (default: toml)\n";
        return false;
    }

    std::string filepath = args[0];
    EFileFormat format   = EFileFormat::TOML;

    if (args.size() > 1) {
        std::string formatStr = args[1];
        StringToLowerInPlace(formatStr);

        if (formatStr == "json")
            format = EFileFormat::JSON;
        else if (formatStr == "toml")
            format = EFileFormat::TOML;
        else {
            std::cerr << CColor::RED << "Unsupported format: " << formatStr << CColor::RESET << "\n";
            std::cerr << "Supported formats: toml, json\n";
            return false;
        }
    }

    if (importEntries(filepath, *m_db, format)) {
        std::cout << CColor::GREEN << "Successfully imported entries from " << filepath << CColor::RESET << "\n";
        return true;
    } else {
        std::cerr << CColor::RED << "Failed to import entries from " << filepath << CColor::RESET << "\n";
        return false;
    }
}

bool CAuthCLI::commandExport(const std::vector<std::string>& args) {
    if (args.empty()) {
        std::cerr << CColor::RED << "Missing argument for export command" << CColor::RESET << "\n";
        std::cerr << "Usage: auth export <file> [format]\n";
        std::cerr << "Supported formats: toml, json (default: toml)\n";
        return false;
    }

    std::string filepath = args[0];
    EFileFormat format   = EFileFormat::TOML;

    if (args.size() > 1) {
        std::string formatStr = args[1];
        StringToLowerInPlace(formatStr);

        if (formatStr == "json")
            format = EFileFormat::JSON;
        else if (formatStr == "toml")
            format = EFileFormat::TOML;
        else {
            std::cerr << CColor::RED << "Unsupported format: " << formatStr << CColor::RESET << "\n";
            std::cerr << "Supported formats: toml, json\n";
            return false;
        }
    }

    auto entries = m_db->getEntries();

    if (entries.empty()) {
        std::cerr << CColor::RED << "No entries to export" << CColor::RESET << "\n";
        return false;
    }

    if (exportEntries(filepath, entries, format)) {
        std::cout << CColor::GREEN << "Successfully exported " << entries.size() << " entries to " << filepath << CColor::RESET << "\n";
        return true;
    } else {
        std::cerr << CColor::RED << "Failed to export entries to " << filepath << CColor::RESET << "\n";
        return false;
    }
}

bool CAuthCLI::commandWipe() {
    auto entries = m_db->getEntries();
    if (entries.empty()) {
        std::cerr << CColor::RED << "No entries to wipe" << CColor::RESET << "\n";
        return false;
    }

    std::string dbPath = GetDatabasePath();
    if (dbPath.empty()) {
        std::cerr << CColor::RED << "Could not find home directory" << CColor::RESET << "\n";
        return false;
    }

    try {
        if (std::filesystem::exists(dbPath))
            std::filesystem::remove(dbPath);

        std::cout << CColor::GREEN << "Database wiped successfully" << CColor::RESET << "\n";
        return true;
    } catch (const std::exception& e) {
        std::cerr << CColor::RED << "Error wiping database: " << e.what() << CColor::RESET << "\n";
        return false;
    }
}