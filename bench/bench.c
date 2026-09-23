/*
 * Benchmark C — heurísticas clássicas de bin packing.
 * Lê instâncias .f64 (little-endian) exportadas pelo binário Rust,
 * roda NF/FF/BF/FFD/BFD e grava CSV no mesmo formato.
 *
 * Compilar: gcc -O2 -o bench_c bench.c -lm
 * Rodar:    ./bench_c <dir_instancias> <saida.csv>
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <time.h>
#include <math.h>

#define EPS 1e-9

typedef struct {
    int bins;
} Solution;

static int cmp_desc(const void *a, const void *b) {
    double x = *(const double *)a, y = *(const double *)b;
    if (x < y) return 1;
    if (x > y) return -1;
    return 0;
}

/* Next Fit — O(n) */
static int next_fit(const double *items, int n) {
    int bins = 1;
    double residual = 1.0;
    for (int i = 0; i < n; i++) {
        if (items[i] <= residual + EPS) {
            residual -= items[i];
        } else {
            bins++;
            residual = 1.0 - items[i];
        }
    }
    return bins;
}

/* First Fit — O(n²) */
static int first_fit(const double *items, int n) {
    double *res = malloc((size_t)n * sizeof(double));
    int nbins = 0;
    for (int i = 0; i < n; i++) {
        int placed = 0;
        for (int b = 0; b < nbins; b++) {
            if (items[i] <= res[b] + EPS) {
                res[b] -= items[i];
                placed = 1;
                break;
            }
        }
        if (!placed) res[nbins++] = 1.0 - items[i];
    }
    free(res);
    return nbins;
}

/* Best Fit — O(n²) */
static int best_fit(const double *items, int n) {
    double *res = malloc((size_t)n * sizeof(double));
    int nbins = 0;
    for (int i = 0; i < n; i++) {
        int best = -1;
        double best_r = 1e18;
        for (int b = 0; b < nbins; b++) {
            if (items[i] <= res[b] + EPS && res[b] < best_r) {
                best = b;
                best_r = res[b];
            }
        }
        if (best >= 0) res[best] -= items[i];
        else res[nbins++] = 1.0 - items[i];
    }
    free(res);
    return nbins;
}

static int first_fit_decreasing(const double *items, int n) {
    double *s = malloc((size_t)n * sizeof(double));
    memcpy(s, items, (size_t)n * sizeof(double));
    qsort(s, (size_t)n, sizeof(double), cmp_desc);
    int r = first_fit(s, n);
    free(s);
    return r;
}

static int best_fit_decreasing(const double *items, int n) {
    double *s = malloc((size_t)n * sizeof(double));
    memcpy(s, items, (size_t)n * sizeof(double));
    qsort(s, (size_t)n, sizeof(double), cmp_desc);
    int r = best_fit(s, n);
    free(s);
    return r;
}

/* microssegundos decorridos (monotônico) */
static double now_us(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e6 + ts.tv_nsec / 1e3;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "uso: %s <dir_instancias> <saida.csv>\n", argv[0]);
        return 1;
    }
    const char *dirpath = argv[1];
    const char *outpath = argv[2];

    DIR *d = opendir(dirpath);
    if (!d) { perror("opendir"); return 1; }

    /* coleta nomes .f64 */
    char names[4096][256];
    int count = 0;
    struct dirent *de;
    while ((de = readdir(d)) != NULL && count < 4096) {
        const char *ext = strrchr(de->d_name, '.');
        if (ext && strcmp(ext, ".f64") == 0) {
            snprintf(names[count], 256, "%s", de->d_name);
            count++;
        }
    }
    closedir(d);

    /* ordena os nomes (mesma ordem do Rust) */
    for (int i = 1; i < count; i++) {
        char key[256];
        snprintf(key, 256, "%s", names[i]);
        int j = i - 1;
        while (j >= 0 && strcmp(names[j], key) > 0) {
            snprintf(names[j + 1], 256, "%s", names[j]);
            j--;
        }
        snprintf(names[j + 1], 256, "%s", key);
    }

    FILE *out = fopen(outpath, "w");
    if (!out) { perror("fopen"); return 1; }
    fprintf(out, "algoritmo,distribuicao,n,rep,bins,lower_bound,razao_vs_lb,tempo_us,soma_tamanhos\n");

    for (int k = 0; k < count; k++) {
        char path[1024];
        snprintf(path, sizeof(path), "%s/%s", dirpath, names[k]);
        FILE *f = fopen(path, "rb");
        if (!f) { perror(path); continue; }
        fseek(f, 0, SEEK_END);
        long sz = ftell(f);
        fseek(f, 0, SEEK_SET);
        int n = (int)(sz / 8);
        double *items = malloc((size_t)sz);
        if (fread(items, 1, (size_t)sz, f) != (size_t)sz) {
            fprintf(stderr, "leitura curta em %s\n", path);
            fclose(f);
            free(items);
            continue;
        }
        fclose(f);

        /* parse do nome: {dist}_n{n}_r{rep} — dist contém '_', quebra
           pela DIREITA */
        char dist[128];
        long n_items = 0, rep = 0;
        char *pn = strrchr(names[k], '_');          /* _r{rep} */
        char *pr = pn;
        if (pn) {
            *pn = '\0';
            rep = strtol(pn + 2, NULL, 10);          /* pula "r" */
            pn = strrchr(names[k], '_');             /* _n{n} */
            if (pn) {
                *pn = '\0';
                n_items = strtol(pn + 2, NULL, 10);
            }
        }
        snprintf(dist, sizeof(dist), "%s", names[k]);
        (void)n_items; /* usa n do arquivo */

        double total = 0.0;
        for (int i = 0; i < n; i++) total += items[i];

        const char *algs[5] = {"NF", "FF", "BF", "FFD", "BFD"};
        for (int a = 0; a < 5; a++) {
            /* aquecimento + medição */
            int bins = 0;
            double t0, t1;
            switch (a) {
            case 0: next_fit(items, n); t0 = now_us(); bins = next_fit(items, n); t1 = now_us(); break;
            case 1: first_fit(items, n); t0 = now_us(); bins = first_fit(items, n); t1 = now_us(); break;
            case 2: best_fit(items, n); t0 = now_us(); bins = best_fit(items, n); t1 = now_us(); break;
            case 3: first_fit_decreasing(items, n); t0 = now_us(); bins = first_fit_decreasing(items, n); t1 = now_us(); break;
            default: best_fit_decreasing(items, n); t0 = now_us(); bins = best_fit_decreasing(items, n); t1 = now_us(); break;
            }
            /* L2 não é recalculado aqui (igual ao Rust) — usa 1 p/ razão;
               o CSV do Rust é a fonte de verdade da razão. */
            fprintf(out, "%s,%s,%d,%ld,%d,0,0.000000,%.1f,%.6f\n",
                    algs[a], dist, n, rep, bins, t1 - t0, total);
        }
        free(items);
    }
    fclose(out);
    fprintf(stderr, "%d arquivos processados (C) -> %s\n", count, outpath);
    return 0;
}
