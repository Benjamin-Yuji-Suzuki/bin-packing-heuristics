// Branch-and-bound exato para bin packing — versão C++.
//
// MESMO ALGORITMO do src/exact.rs (Rust) e do bnb.c:
//   - incumbent inicial = First Fit Decreasing;
//   - itens em ordem DECRESCENTENTE;
//   - DFS com Poda 1 (incumbent) e Poda 2 (lower bound ceil(R - F)).
//
// Uso: bnb_cpp <dir_instancias> <saida.csv> [n_max]
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <dirent.h>
#include <fstream>
#include <string>
#include <vector>

constexpr double EPS = 1e-9;

static std::vector<double> g_itens, g_resto;
static int g_n, g_melhor;

static int ffd_bins(const std::vector<double> &it) {
    std::vector<double> s = it;
    std::sort(s.begin(), s.end(), std::greater<double>());
    std::vector<double> res(s.size());
    int nb = 0;
    for (double x : s) {
        bool placed = false;
        for (int b = 0; b < nb; b++) {
            if (x <= res[b] + EPS) { res[b] -= x; placed = true; break; }
        }
        if (!placed) res[nb++] = 1.0 - x;
    }
    return nb;
}

static void dfs(int i, std::vector<double> &residuos) {
    int nb = (int)residuos.size();
    if (nb >= g_melhor) return;                       // Poda 1
    if (i == g_n) { g_melhor = nb; return; }

    double r = g_resto[i];
    double f = 0.0;
    for (double x : residuos) f += x;
    int novos = (r > f) ? (int)std::ceil((r - f) - EPS) : 0;   // Poda 2
    if (nb + novos >= g_melhor) return;

    double x = g_itens[i];
    for (int b = 0; b < nb; b++) {
        if (x <= residuos[b] + EPS) {
            residuos[b] -= x;
            dfs(i + 1, residuos);
            residuos[b] += x;
        }
    }
    residuos.push_back(1.0 - x);
    dfs(i + 1, residuos);
    residuos.pop_back();
}

static double now_ms() {
    timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e3 + ts.tv_nsec / 1e6;
}

static int optimal_bins(const std::vector<double> &it, double &ms) {
    if (it.empty()) return 0;
    g_melhor = ffd_bins(it);
    g_itens = it;
    std::sort(g_itens.begin(), g_itens.end(), std::greater<double>());
    g_n = (int)g_itens.size();
    g_resto.assign(g_n + 1, 0.0);
    for (int i = g_n - 1; i >= 0; i--) g_resto[i] = g_resto[i + 1] + g_itens[i];

    std::vector<double> residuos;
    residuos.reserve(g_n);
    double t0 = now_ms();
    dfs(0, residuos);
    ms = now_ms() - t0;
    return g_melhor;
}

static std::vector<double> ler_f64(const std::string &path) {
    std::ifstream f(path, std::ios::binary);
    f.seekg(0, std::ios::end);
    long sz = f.tellg();
    f.seekg(0);
    std::vector<double> v(sz / 8);
    f.read(reinterpret_cast<char *>(v.data()), sz);
    return v;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        std::fprintf(stderr, "uso: %s <dir> <saida.csv> [n_max]\n", argv[0]);
        return 1;
    }
    const char *dirpath = argv[1];
    const char *outpath = argv[2];
    int n_max = (argc > 3) ? std::atoi(argv[3]) : 60;

    std::vector<std::string> names;
    DIR *d = opendir(dirpath);
    if (!d) { perror("opendir"); return 1; }
    struct dirent *de;
    while ((de = readdir(d)) != nullptr) {
        std::string nme = de->d_name;
        if (nme.size() > 4 && nme.compare(nme.size() - 4, 4, ".f64") == 0)
            names.push_back(nme);
    }
    closedir(d);
    std::sort(names.begin(), names.end());

    FILE *out = fopen(outpath, "w");
    if (!out) { perror("fopen"); return 1; }
    std::fprintf(out, "distribuicao,n,rep,optimal,bnb_ms\n");

    int feitos = 0;
    for (const std::string &name : names) {
        std::string path = std::string(dirpath) + "/" + name;
        std::vector<double> items = ler_f64(path);
        int n = (int)items.size();

        std::string stem = name.substr(0, name.size() - 4);
        size_t rp = stem.rfind("_r");
        long rep = std::strtol(stem.c_str() + rp + 2, nullptr, 10);
        stem = stem.substr(0, rp);
        size_t np = stem.rfind("_n");
        std::string dist = stem.substr(0, np);

        if (n > n_max) continue;
        double ms;
        int opt = optimal_bins(items, ms);
        std::fprintf(out, "%s,%d,%ld,%d,%.4f\n", dist.c_str(), n, rep, opt, ms);
        feitos++;
    }
    fclose(out);
    std::fprintf(stderr, "%d instancias com n <= %d resolvidas (C++) -> %s\n",
                 feitos, n_max, outpath);
    return 0;
}