#pragma once

#include <random>

thread_local std::mt19937 generator_;
thread_local std::uniform_int_distribution<uint64_t> uint64_distribution_;
uint64_t randInt64() { return uint64_distribution_(generator_); }