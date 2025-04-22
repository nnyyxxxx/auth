#include <catch2/catch_test_macros.hpp>
#include "../src/core/Totp.hpp"
#include <string>

TEST_CASE("TOTP constructor sets correct default values", "[totp]") {
    CTotp totp("ABCDEFGHIJKLMNOP");

    REQUIRE_NOTHROW(totp.generate());
}

TEST_CASE("TOTP constructor with custom parameters", "[totp]") {
    CTotp totp("ABCDEFGHIJKLMNOP", 8, 60);
    REQUIRE_NOTHROW(totp.generate());
}

TEST_CASE("TOTP handles empty secret", "[totp]") {
    CTotp totp("");
    REQUIRE(totp.generate() == "Invalid key");
}

TEST_CASE("TOTP handles invalid Base32 characters in secret", "[totp]") {
    CTotp totp("!@#$%^&*()");
    REQUIRE(totp.generate() == "Invalid key");
}

TEST_CASE("TOTP handles valid Base32 secret", "[totp]") {
    CTotp             totp("JBSWY3DPEHPK3PXP");
    const std::string code = totp.generate();

    REQUIRE(code.length() == 6);
    REQUIRE(std::all_of(code.begin(), code.end(), [](char c) { return std::isdigit(c); }));
}

TEST_CASE("TOTP with 8 digit output", "[totp]") {
    CTotp             totp("JBSWY3DPEHPK3PXP", 8);
    const std::string code = totp.generate();

    REQUIRE(code.length() == 8);
    REQUIRE(std::all_of(code.begin(), code.end(), [](char c) { return std::isdigit(c); }));
}

TEST_CASE("TOTP handles spaces and dashes in secret", "[totp]") {
    CTotp totp1("JBSWY3DPEHPK3PXP");
    CTotp totp2("JBSW Y3DP-EHPK 3PXP");

    REQUIRE(totp1.generate() == totp2.generate());
}