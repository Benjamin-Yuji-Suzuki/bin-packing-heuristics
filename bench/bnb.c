/*
 * Branch-and-bound exato para bin packing — versão C.
 *
 * MESMO ALGORITMO do src/exact.rs do Rust:
 *   - incumbent inicial = First Fit Decreasing (upper bound);
 *   - itens ordenados DECRESCENTEMENTE;
 *   - DFS: tenta encaixar o item em cada bin aberto (ordem de abertura)
 *     ou abre um bin novo;
 *   - Poda 1: incumbent (residuos.n >= melhor);
 *   - Poda 2: lower bound ceil(R - F), onde R = soma dos itens
 *     restantes e F = folga total dos bins abertos.
 *
 * A comparacao entre linguagens so e justa porque o TRABALHO e o mesmo:
 * mesma ordem de exploracao, mesmas podas, mesmo incumbent inicial.
 *
 * Uso: bnb_c <dir_instancias> <saida.csv> [tamanho_maximo]
 *   As instancias sao as .f64 exportadas pelo Rust; so processa as de
 *   tamanho <= tamanho_maximo (padrao 60: alem disso o B&B estoura).
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <time.h>
#include <math.h>

#define EPS 1e-9

static int cmp_desc(const void *a, const void *b) {
    double x = *(const double *)a, y = *(const double *)b;
    if (x < y) return 1;
    if (x > y) return -1;
    return 0;
}

/* First Fit Decreasing — incumbent inicial (mesmo do Rust) */
static int ffd_bins(const double *it, int n) {
    double *s = malloc((size_t)n * sizeof(double));
    memcpy(s, it, (size_t)n * sizeof(double));
    qsort(s, (size_t)n, sizeof(double), cmp_desc);
    double *res = malloc((size_t)n * sizeof(double));
    int nb = 0;
    for (int i = 0; i < n; i++) {
        int placed = 0;
        for (int b = 0; b < nb; b++) {
            if (s[i] <= res[b] + EPS) { res[b] -= s[i]; placed = 1; break; }
        }
        if (!placed) res[nb++] = 1.0 - s[i];
    }
    free(res); free(s);
    return nb;
}

static double *g_itens, *g_resto;
static int g_n;
static int g_melhor;

static void dfs(int i, double *residuos, int nb) {
    if (nb >= g_melhor) return;                 /* Poda 1 */
    if (i == g_n) { g_melhor = nb; return; }

    /* Poda 2: lower bound de bins ainda necessarios */
    double r = g_resto[i];
    double f = 0.0;
    for (int b = 0; b < nb; b++) f += residuos[b];
    int novos = (r > f) ? (int)ceil((r - f) - EPS) : 0;
    if (nb + novos >= g_melhor) return;

    double x = g_itens[i];
    for (int b = 0; b < nb; b++) {
        if (x <= residuos[b] + EPS) {
            residuos[b] -= x;
            dfs(i + 1, residuos, nb);
            residuos[b] += x;
        }
    }
    residuos[nb] = 1.0 - x;
    dfs(i + 1, residuos, nb + 1);
}

static int optimal_bins(const double *it, int n, double *ms) {
    if (n == 0) return 0;
    double *itens = malloc((size_t)n * sizeof(double));
    memcpy(itens, it, (size_t)n * sizeof(double));

    int melhor = ffd_bins(it, n);

    qsort(itens, (size_t)n, sizeof(double), cmp_desc);

    g_n = n;
    g_itens = itens;
    g_resto = malloc((size_t)(n + 1) * sizeof(double));
    g_resto[n] = 0.0;
    for (int i = n - 1; i >= 0; i--) g_resto[i] = g_resto[i + 1] + itens[i];
    g_melhor = melhor;

    double *residuos = malloc((size_t)n * sizeof(double));
    struct timespec t0, t1;
    clock_gettime(CLOCK_MONOTONIC, &t0);
    dfs(0, residuos, 0);
    clock_gettime(CLOCK_MONOTONIC, &t1);
    *ms = (t1.tv_sec - t0.tv_sec) * 1e3 + (t1.tv_nsec - t0.tv_nsec) / 1e6;

    free(residuos); free(g_resto); free(itens);
    return g_melhor;
}

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e3 + ts.tv_nsec / 1e6;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "uso: %s <dir> <saida.csv> [n_max]\n", argv[0]);
        return 1;
    }
    const char *dirpath = argv[1], *outpath = argv[2];
    int n_max = (argc > 3) ? atoi(argv[3]) : 60;

    DIR *d = opendir(dirpath);
    if (!d) { perror("opendir"); return 1; }
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
    for (int i = 1; i < count; i++) {           /* insertion sort, como no Rust */
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
    fprintf(out, "distribuicao,n,rep,optimal,bnb_ms\n");

    int feitos = 0;
    for (int k = 0; k < count; k++) {
        char path[1024], dist[128];
        long n_items = 0, rep = 0;
        snprintf(path, sizeof(path), "%s/%s", dirpath, names[k]);
        FILE *f = fopen(path, "rb");
        if (!f) { perror(path); continue; }
        fseek(f, 0, SEEK_END); long sz = ftell(f); fseek(f, 0, SEEK_SET);
        int n = (int)(sz / 8);
        double *items = malloc((size_t)sz);
        if (fread(items, 1, (size_t)sz, f) != (size_t)sz) { fclose(f); free(items); continue; }
        fclose(f);

        char *pn = strrchr(names[k], '_');
        if (pn) {
            *pn = '\0';
            rep = strtol(pn + 2, NULL, 10);
            pn = strrchr(names[k], '_');
            if (pn) { *pn = '\0'; n_items = strtol(pn + 2, NULL, 10); }
        }
        snprintf(dist, sizeof(dist), "%s", names[k]);
        (void)n_items;

        if (n > n_max) { free(items); continue; }

        double ms;
        int opt = optimal_bins(items, n, &ms);
        fprintf(out, "%s,%d,%ld,%d,%.4f\n", dist, n, rep, opt, ms);
        feitos++;
        free(items);
    }
    fclose(out);
    fprintf(stderr, "%d instancias com n <= %d resolvidas (C) -> %s\n", feitos, n_max, outpath);
    return 0;
}