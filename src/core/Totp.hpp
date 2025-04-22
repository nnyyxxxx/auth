#pragma once

#include <cstdint>
#include <string>

class CTotp {
  public:
    CTotp(const std::string& secret, uint32_t digits = 6, uint32_t period = 30);
    std::string generate() const;

  private:
    std::string m_secret;
    uint32_t    m_digits = 6;
    uint32_t    m_period = 30;
};