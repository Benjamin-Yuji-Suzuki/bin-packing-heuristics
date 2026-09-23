// Benchmark C++ — heurísticas clássicas de bin packing.
// Lê instâncias .f64 (little-endian) exportadas pelo binário Rust,
// roda NF/FF/BF/FFD/BFD e grava CSV no mesmo formato.
//
// Compilar: g++ -O2 -std=c++17 -o bench_cpp bench.cpp
// Rodar:    ./bench_cpp <dir_instancias> <saida.csv>
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

static constexpr double kEps = 1e-9;

int NextFit(const std::vector<double> &items) {
    int bins = 1;
    double residual = 1.0;
    for (double x : items) {
        if (x <= residual + kEps) {
            residual -= x;
        } else {
            ++bins;
            residual = 1.0 - x;
        }
    }
    return bins;
}

int FirstFit(const std::vector<double> &items) {
    std::vector<double> res;
    res.reserve(items.size());
    for (double x : items) {
        bool placed = false;
        for (size_t b = 0; b < res.size(); ++b) {
            if (x <= res[b] + kEps) {
                res[b] -= x;
                placed = true;
                break;
            }
        }
        if (!placed) res.push_back(1.0 - x);
    }
    return static_cast<int>(res.size());
}

int BestFit(const std::vector<double> &items) {
    std::vector<double> res;
    res.reserve(items.size());
    for (double x : items) {
        int best = -1;
        double best_r = 1e18;
        for (size_t b = 0; b < res.size(); ++b) {
            if (x <= res[b] + kEps && res[b] < best_r) {
                best = static_cast<int>(b);
                best_r = res[b];
            }
        }
        if (best >= 0) res[best] -= x;
        else res.push_back(1.0 - x);
    }
    return static_cast<int>(res.size());
}

int FirstFitDecreasing(std::vector<double> items) {
    std::sort(items.begin(), items.end(), std::greater<double>());
    return FirstFit(items);
}

int BestFitDecreasing(std::vector<double> items) {
    std::sort(items.begin(), items.end(), std::greater<double>());
    return BestFit(items);
}

template <typename F>
long long MeasureMicros(F &&fn, int &bins) {
    auto t0 = std::chrono::steady_clock::now();
    bins = fn();
    auto t1 = std::chrono::steady_clock::now();
    return std::chrono::duration_cast<std::chrono::microseconds>(t1 - t0).count();
}

int main(int argc, char **argv) {
    if (argc < 3) {
        std::fprintf(stderr, "uso: %s <dir_instancias> <saida.csv>\n", argv[0]);
        return 1;
    }
    std::string dirpath = argv[1], outpath = argv[2];

    // lista .f64
    std::vector<std::string> names;
    std::string cmd = "ls " + dirpath + "/*.f64 2>/dev/null";
    FILE *pipe = popen(cmd.c_str(), "r");
    if (!pipe) { perror("popen"); return 1; }
    char buf[512];
    while (std::fgets(buf, sizeof(buf), pipe)) {
        std::string s(buf);
        while (!s.empty() && (s.back() == '\n' || s.back() == '\r')) s.pop_back();
        // só o nome do arquivo, sem o diretório
        auto slash = s.find_last_of('/');
        if (slash != std::string::npos) s = s.substr(slash + 1);
        if (!s.empty()) names.push_back(s);
    }
    pclose(pipe);
    std::sort(names.begin(), names.end());

    std::ofstream out(outpath);
    out << "algoritmo,distribuicao,n,rep,bins,lower_bound,razao_vs_lb,tempo_us,soma_tamanhos\n";

    for (const auto &name : names) {
        std::ifstream f(dirpath + "/" + name, std::ios::binary);
        if (!f) continue;
        f.seekg(0, std::ios::end);
        size_t sz = static_cast<size_t>(f.tellg());
        f.seekg(0, std::ios::beg);
        size_t n = sz / 8;
        std::vector<double> items(n);
        f.read(reinterpret_cast<char *>(items.data()), static_cast<std::streamsize>(sz));
        f.close();

        // {dist}_n{n}_r{rep} — dist contém '_', quebra pela DIREITA
        std::string stem = name.substr(0, name.find_last_of('.'));
        auto p1 = stem.find_last_of('_');           // _r{rep}
        long rep = std::strtol(stem.c_str() + p1 + 2, nullptr, 10);
        auto p2 = stem.find_last_of('_', p1 - 1);   // _n{n}
        std::string dist = stem.substr(0, p2);

        double total = 0.0;
        for (double x : items) total += x;

        struct Result { const char *name; int bins; long long us; };
        Result results[5];
        // aquecimento + medição (bins preenchido por referência ANTES da
        // atribuição — não usar inicializador com literal 0 aqui)
        auto run = [&](int i, const char *nm, auto &&fn) {
            fn(); // aquecimento
            results[i].name = nm;
            results[i].us = MeasureMicros(fn, results[i].bins);
        };
        run(0, "NF",  [&] { return NextFit(items); });
        run(1, "FF",  [&] { return FirstFit(items); });
        run(2, "BF",  [&] { return BestFit(items); });
        run(3, "FFD", [&] { return FirstFitDecreasing(items); });
        run(4, "BFD", [&] { return BestFitDecreasing(items); });

        for (const auto &r : results) {
            out << r.name << "," << dist << "," << n << "," << rep << ","
                << r.bins << ",0,0.000000," << r.us << "," << total << "\n";
        }
    }
    std::fprintf(stderr, "%zu arquivos processados (C++) -> %s\n", names.size(), outpath.c_str());
    return 0;
}
