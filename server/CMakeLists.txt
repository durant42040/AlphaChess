cmake_minimum_required(VERSION 3.28)
project(server)

set(CMAKE_CXX_STANDARD 17)

add_executable(server src/main.cpp src/stockfish.cpp ../engine/engine.cpp ../engine/chessboard.cpp ../engine/move_generator.cpp)    
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../engine)
