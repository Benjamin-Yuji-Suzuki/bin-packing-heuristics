#!/usr/bin/env python3
"""Mede o uso INSTANTANEO de CPU por processo.

`ps pcpu` e a media de uso desde o inicio do processo — para um processo
que roda ha 23 horas, esse numero nao descreve o que esta acontecendo
agora. Para benchmark, so interessa o uso instantaneo.

Le /proc/<pid>/stat duas vezes com um intervalo curto e calcula o delta,
que e o uso real no periodo. Uso de referencia (idle total) vem de
/proc/stat, entao o resultado e normalizado pelo numero de CPUs.

Uso: python3 scripts/cpu_instantanea.py [intervalo_segundos]
"""
import sys
import time
from pathlib import Path

CLK_TCK = 100.0  # getconf CLK_TCK no Linux x86_64 = 100
N_CPUS = None


def n_cpus():
    return len(
        [l for l in Path("/proc/stat").read_text().splitlines() if l.startswith("cpu") and l[3].isdigit()]
    )


def cpu_total():
    """(idle+total) global, em jiffies."""
    for line in Path("/proc/stat").read_text().splitlines():
        if line.startswith("cpu "):
            f = [int(x) for x in line.split()[1:]]
            idle = f[3] + (f[4] if len(f) > 4 else 0)  # idle + iowait
            return sum(f), idle
    return 0, 0


def snap():
    """(pid -> (nome, utime+stime)) para todos os processos."""
    out = {}
    for p in Path("/proc").iterdir():
        if not p.name.isdigit():
            continue
        try:
            stat = (p / "stat").read_text()
            # campo 2 (comm) pode conter espacos e parenteses: quebra pelo ultimo ')'
            comm = stat[stat.index("(") + 1 : stat.rindex(")")]
            rest = stat[stat.rindex(")") + 2 :].split()
            utime, stime = int(rest[11]), int(rest[12])
            out[int(p.name)] = (comm, utime + stime)
        except (OSError, ValueError, IndexError):
            continue
    return out


def main():
    global N_CPUS
    intervalo = float(sys.argv[1]) if len(sys.argv) > 1 else 1.0
    N_CPUS = n_cpus()

    t0, i0 = cpu_total()
    s0 = snap()
    time.sleep(intervalo)
    t1, i1 = cpu_total()
    s1 = snap()

    total_delta = (t1 - t0) - (i1 - i0)  # jiffies de trabalho real
    if total_delta <= 0:
        print("  (intervalo curto demais para medir; carga praticamente zero)")
        return

    print(f"  CPUs: {N_CPUS} | uso total do sistema: {100 * total_delta / (t1 - t0):.1f}%")
    print()
    print("  Processos por uso INSTANTANEO de CPU:")
    linhas = []
    for pid, (comm, j) in s1.items():
        if pid in s0:
            d = j - s0[pid][1]
            if d <= 0:
                continue
            pct = 100.0 * d / total_delta  # % de 1 core
            if pct >= 0.5:
                linhas.append((pct, pid, comm))
    for pct, pid, comm in sorted(linhas, reverse=True)[:8]:
        print(f"    {pct:5.1f}%  pid {pid:<8} {comm}")


if __name__ == "__main__":
    main()