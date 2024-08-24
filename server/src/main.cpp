#include <iostream>
#include <string>
#include <thread>
#include <vector>
#include <sstream>
#include <netinet/in.h>
#include <unistd.h>
#include "stockfish.h"
#include "engine.h"

void handle_make_move(int client_socket, std::string move) {
    if (move.empty()) {
        std::string response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
        send(client_socket, response.c_str(), response.length(), 0);
        close(client_socket);
        return;
    }
    
    if (!act(move)) {
        std::string response = "HTTP/1.1 400 Bad Request\r\n"
        "Content-Type: application/json\r\n"
        "Access-Control-Allow-Origin: *\r\n"
        "Content-Length: " + std::to_string(26 + move.size()) + "\r\n"
        "\r\n{\"error\":\"Illegal move: " + move + "\"}";
        
        send(client_socket, response.c_str(), response.length(), 0);
        close(client_socket);
        return;
    }

    const std::string response = 
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: application/json\r\n"
        "Access-Control-Allow-Origin: *\r\n" 
        "\r\n"
        "{\"board\":\"" + get_board() + "\",\"isCheck\":" + std::to_string(is_check()) + "}";

    send(client_socket, response.c_str(), response.size(), 0);
    close(client_socket);
}

void handle_reset(int client_socket) {
    std::cout << "reset" << std::endl;
    reset_engine();
    const std::string response = 
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: text/plain\r\n"
        "Content-Length: 2\r\n"
        "Access-Control-Allow-Origin: *\r\n"
        "\r\n"
        "OK";

    send(client_socket, response.c_str(), response.size(), 0);
    close(client_socket);
}

void handle_genmove(int client_socket) {
    std::vector<std::string> moves_string;
    for (const auto& move : moves) {
        moves_string.push_back(move.to_string());
    }
    const std::string move = generate_move_stockfish(moves_string);
    act(move);

    const std::string response = 
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: application/json\r\n"
        "Access-Control-Allow-Origin: *\r\n"
        "\r\n{\"move\":\"" + move + "\",\"board\":\"" + get_board() + "\",\"isCheck\":" + std::to_string(is_check()) + "}";

    send(client_socket, response.c_str(), response.size(), 0);
    close(client_socket);
}

void start_server(int port) {
    int server_socket = socket(AF_INET, SOCK_STREAM, 0);
    if (server_socket == -1) {
        std::cerr << "Failed to create socket" << std::endl;
        return;
    }

    sockaddr_in server_addr;
    server_addr.sin_family = AF_INET;
    server_addr.sin_addr.s_addr = INADDR_ANY;
    server_addr.sin_port = htons(port);

    if (bind(server_socket, (struct sockaddr*)&server_addr, sizeof(server_addr)) == -1) {
        std::cerr << "Failed to bind socket" << std::endl;
        close(server_socket);
        return;
    }

    if (listen(server_socket, 10) == -1) {
        std::cerr << "Failed to listen on socket" << std::endl;
        close(server_socket);
        return;
    }

    try {
        init_engine();
    } catch (const std::exception& e) {
        std::cerr << e.what() << std::endl;
        close(server_socket);
        return;
    }

    std::cout << "Server is listening on port " << port << std::endl;
    
    while (true) {
        int client_socket = accept(server_socket, nullptr, nullptr);
        if (client_socket == -1) {
            std::cerr << "Failed to accept client connection" << std::endl;
            continue;
        }

        char buffer[1024] = {0};
        read(client_socket, buffer, 1024);

        std::string request(buffer);

        if (request.find("GET /genmove") != std::string::npos) {
            std::thread(handle_genmove, client_socket).detach();
        } else if (request.find("GET /make_move") != std::string::npos) {
            std::string query = request.substr(request.find("?") + 1);
            std::string move;

            if (query.find("move=") != std::string::npos) {
                size_t start = query.find("move=") + 5;
                size_t end = query.find(' ', start);
                move = query.substr(start, end - start);
            }
            std::thread(handle_make_move, client_socket, move).detach();
        } else if (request.find("GET /reset") != std::string::npos) {
            std::thread(handle_reset, client_socket).detach();
        } else if (request.find("GET /game") != std::string::npos) {
            std::string response = "HTTP/1.1 200 OK\r\n"
            "Content-Type: application/json\r\n"
            "Access-Control-Allow-Origin: *\r\n"
            "Content-Length: " + std::to_string(get_game_state().length() + 19) + "\r\n\r\n" 
            "{ \"gameState\": \"" + get_game_state() + "\" }"; 
            
            
            send(client_socket, response.c_str(), response.length(), 0);
            close(client_socket);
        } else {
            std::string response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            send(client_socket, response.c_str(), response.length(), 0);
            close(client_socket);
        }
    }

    close(server_socket);
}

int main() {
    int port = 4000;
    start_server(port);
    return 0;
}
