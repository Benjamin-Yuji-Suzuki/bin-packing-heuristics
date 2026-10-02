/* Auditoria de equivalencia de trabalho — versao C.
 *
 * Conta as MESMAS operacoes elementares que src/bin/auditoria_trabalho.rs
 * (comparacoes e subtracoes sobre o residuo), sobre as MESMAS
 * instancias. O script Python compara as duas contagens.
 *
 * Compilar: gcc -O2 -o aud_c aud_c.c -lm
 * Usar:     ./aud_c <dir_instancias> > contagens_c.csv
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>

#define EPS 1e-9

static void nf_conta(const double *it, int n, long *c, long *s, int *bins) {
    int nb = 1; double r = 1.0; *c = 0; *s = 0;
    for (int i = 0; i < n; i++) {
        (*c)++;
        if (it[i] <= r + EPS) { r -= it[i]; (*s)++; }
        else { nb++; r = 1.0 - it[i]; }
    }
    *bins = nb;
}

static void ff_conta(const double *it, int n, long *c, long *s, int *bins) {
    double *res = malloc((size_t)n * sizeof(double));
    int nb = 0; *c = 0; *s = 0;
    for (int i = 0; i < n; i++) {
        int pl = 0;
        for (int b = 0; b < nb; b++) {
            (*c)++;
            if (it[i] <= res[b] + EPS) { res[b] -= it[i]; (*s)++; pl = 1; break; }
        }
        if (!pl) res[nb++] = 1.0 - it[i];
    }
    *bins = nb; free(res);
}

static void bf_conta(const double *it, int n, long *c, long *s, int *bins) {
    double *res = malloc((size_t)n * sizeof(double));
    int nb = 0; *c = 0; *s = 0;
    for (int i = 0; i < n; i++) {
        int best = -1; double br = 1e18;
        for (int b = 0; b < nb; b++) {
            (*c)++;
            if (it[i] <= res[b] + EPS && res[b] < br) { best = b; br = res[b]; }
        }
        if (best >= 0) { res[best] -= it[i]; (*s)++; }
        else res[nb++] = 1.0 - it[i];
    }
    *bins = nb; free(res);
}

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "uso: %s <dir>\n", argv[0]); return 1; }
    DIR *d = opendir(argv[1]);
    if (!d) { perror("opendir"); return 1; }
    char names[4096][256]; int count = 0;
    struct dirent *de;
    while ((de = readdir(d)) != NULL) {
        const char *e = strrchr(de->d_name, '.');
        if (e && strcmp(e, ".f64") == 0 && count < 4096) {
            snprintf(names[count], 256, "%s", de->d_name); count++;
        }
    }
    closedir(d);
    for (int i = 1; i < count; i++) {
        char k[256]; snprintf(k, 256, "%s", names[i]); int j = i - 1;
        while (j >= 0 && strcmp(names[j], k) > 0) { snprintf(names[j+1], 256, "%s", names[j]); j--; }
        snprintf(names[j+1], 256, "%s", k);
    }
    printf("alg,dist,n,comp,sub,bins\n");
    for (int k = 0; k < count; k++) {
        char path[1100];
        snprintf(path, sizeof(path), "%s/%s", argv[1], names[k]);
        FILE *f = fopen(path, "rb"); if (!f) continue;
        fseek(f, 0, SEEK_END); long sz = ftell(f); fseek(f, 0, SEEK_SET);
        int n = (int)(sz / 8);
        double *it = malloc((size_t)sz);
        if (fread(it, 1, (size_t)sz, f) != (size_t)sz) { fclose(f); free(it); continue; }
        fclose(f);
        char dist[128];
        char *pn = strrchr(names[k], '_');
        if (pn) { *pn = '\0'; pn = strrchr(names[k], '_'); if (pn) *pn = '\0'; }
        snprintf(dist, sizeof(dist), "%s", names[k]);
        long c, s; int b;
        nf_conta(it, n, &c, &s, &b); printf("NF,%s,%d,%ld,%ld,%d\n", dist, n, c, s, b);
        ff_conta(it, n, &c, &s, &b); printf("FF,%s,%d,%ld,%ld,%d\n", dist, n, c, s, b);
        bf_conta(it, n, &c, &s, &b); printf("BF,%s,%d,%ld,%ld,%d\n", dist, n, c, s, b);
        free(it);
    }
    return 0;
}