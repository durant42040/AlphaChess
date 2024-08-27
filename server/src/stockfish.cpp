#include "stockfish.h"
#include <array>
#include <cstdio>
#include <iostream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <sys/socket.h>
#include <unistd.h>

std::string generate_move_stockfish(std::vector<std::string> &moves) {
    std::string input = "position startpos moves";
    for (const auto &move : moves) {
        input += " " + move;
    }

    std::unique_ptr<FILE, decltype(&pclose)> pipe(popen("stockfish", "r+"),
                                                  pclose);
    if (!pipe) {
        throw std::runtime_error("popen() failed!");
    }

    fprintf(pipe.get(), "%s\n", input.c_str());
    fflush(pipe.get());

    fprintf(pipe.get(), "go depth 20\n");
    fflush(pipe.get());

    std::array<char, 128> buffer;
    std::string result;
    while (fgets(buffer.data(), buffer.size(), pipe.get()) != nullptr) {
        result += buffer.data();
        if (result.find("bestmove") != std::string::npos) {
            break;
        }
    }

    std::string bestmove;
    std::istringstream iss(result);
    std::string line;
    while (std::getline(iss, line)) {
        if (line.find("bestmove") != std::string::npos) {
            size_t start = line.find("bestmove") + 9;
            size_t end = line.find(' ', start);
            bestmove = line.substr(start, end - start);
            break;
        }
    }

    return bestmove;
}