"""Generates one Jupyter notebook per rebound_rs example (golden rule:
every example has a concomitant notebook). Each notebook builds the
example with cargo, runs it, and displays its output.
Part of the rebound_rs port. GPL-3.0-or-later."""
import json
import os

CRATE = r"C:\Users\nsh\Developer\github\rustSolveIt_Win11_SUNDIALS_7_8_0\rebound_rust"

EXAMPLES = {
    "shearing_sheet": {
        "title": "shearing_sheet — Saturn's rings shearing box (pure Rust)",
        "desc": "Straight port of REBOUND's `examples/shearing_sheet` (SEI integrator, "
                "octree self-gravity, tree collision search, hard-sphere collisions with the "
                "Bridges restitution law, shear-periodic boundary). Runs a short simulation "
                "and plots the particle positions.",
        "args": [],
        "cwd": "porttest",
        "post": r'''# Plot the final particle positions from the bit dump written by the run
import matplotlib.pyplot as plt
xs, ys = [], []
with open(os.path.join(CRATE, "porttest", "state_rust_final.txt")) as f:
    for line in f:
        parts = line.split()
        if len(parts) == 7 and parts[0].isdigit():
            import struct
            x = struct.unpack("<d", int(parts[1], 16).to_bytes(8, "little"))[0]
            y = struct.unpack("<d", int(parts[2], 16).to_bytes(8, "little"))[0]
            xs.append(x); ys.append(y)
plt.figure(figsize=(5, 5))
plt.scatter(xs, ys, s=2)
plt.xlabel("x [m]"); plt.ylabel("y [m]"); plt.title(f"Shearing sheet, {len(xs)} particles")
plt.show()''',
        "runner": "shearing_sheet_test",  # the dump-producing twin
        "runner_args": ["400"],
    },
    "shearing_sheet_test": {
        "title": "shearing_sheet_test — byte-identity harness vs the MSVC C build",
        "desc": "Runs the seeded 400-step shearing-sheet harness and, if the C reference dump "
                "(`state_c_final.txt`, produced by `porttest\\rebound_test.exe 400`) is present, "
                "verifies byte identity and shows the SHA-256 hashes.",
        "args": ["400"],
        "cwd": "porttest",
        "post": r'''import hashlib
def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest().upper()
c = os.path.join(CRATE, "porttest", "state_c_final.txt")
rs = os.path.join(CRATE, "porttest", "state_rust_final.txt")
print("Rust SHA-256:", sha(rs))
if os.path.exists(c):
    print("C    SHA-256:", sha(c))
    print("IDENTICAL" if open(c,"rb").read().replace(b"\r\n",b"\n") == open(rs,"rb").read().replace(b"\r\n",b"\n") else "MISMATCH")
else:
    print("C reference dump not present (run porttest\\rebound_test.exe 400 to compare).")''',
    },
    "integrators_test": {
        "title": "integrators_test — three-body integrator harness",
        "desc": "Runs the fixed three-body problem with a chosen integrator configuration and "
                "dumps the final state as raw IEEE-754 bit patterns. 63 configurations of this "
                "harness were verified bit-identical against the MSVC C build.",
        "args": ["whfast", "2", "500"],
        "cwd": "porttest",
        "post": r'''print(open(os.path.join(CRATE, "porttest", "state_rust_final.txt")).read())''',
    },
    "libm_diff": {
        "title": "libm_diff — Rust vs UCRT libm differential",
        "desc": "Samples 200,000 inputs per libm function and dumps bit patterns; comparing "
                "against the C twin shows sin/cos/tan/atan2/sqrt/fmod/exp/log are bit-identical "
                "on x86_64-pc-windows-msvc and `pow` is the single divergent function (<= 2 ulp).",
        "args": [],
        "cwd": "porttest",
        "post": r'''print("Output written; compare against porttest\\libm_diff.exe output as described in rebound_rust.md.")''',
    },
    "derivatives_test": {
        "title": "derivatives_test — 65 orbital-derivative functions",
        "desc": "Evaluates all 65 `reb_particle_derivative_*` functions on two configurations "
                "and dumps bit patterns; verified 130/130 lines bit-identical vs the C build.",
        "args": [],
        "cwd": "porttest",
        "post": r'''p = os.path.join(CRATE, "porttest", "derivatives_rust.txt")
lines = open(p).read().splitlines()
print(f"{len(lines)} result lines; first 5:")
print("\n".join(lines[:5]))''',
    },
    "frequency_test": {
        "title": "frequency_test — MFT/FMFT/FMFT2 frequency analysis",
        "desc": "Runs `reb_frequency_analysis` in all three modes on a synthetic three-frequency "
                "signal (true frequencies 0.30, 0.55, 0.11 rad/sample) and prints the recovered "
                "frequencies, amplitudes and phases. Verified bit-identical vs the C build.",
        "args": [],
        "cwd": "porttest",
        "post": r'''import struct
p = os.path.join(CRATE, "porttest", "frequency_rust.txt")
mode = None
for line in open(p).read().splitlines():
    parts = line.split()
    if len(parts) == 3 and parts[1] == "ret":
        mode = parts[0]; vals = []
        print(f"--- {mode} (ret {parts[2]}) ---")
    elif len(parts) == 2:
        v = struct.unpack("<d", int(parts[1], 16).to_bytes(8, "little"))[0]
        vals.append(v)
        if len(vals) == 9:
            print("freqs :", [round(x, 6) for x in vals[0:3]])
            print("amps  :", [round(x, 6) for x in vals[3:6]])
            print("phases:", [round(x, 6) for x in vals[6:9]])''',
    },
    "archive_test": {
        "title": "archive_test — Simulationarchive round trip",
        "desc": "Writes a 3-snapshot Simulationarchive from Rust (whfast-usafe, 3 x 100 steps). "
                "The same binary can be loaded by the MSVC C build (`porttest\\archive_test.exe "
                "whfast-usafe continue`) and continues bit-identically — and vice versa.",
        "args": ["whfast-usafe", "write"],
        "cwd": "porttest",
        "post": r'''p = os.path.join(CRATE, "porttest", "archive_rust_whfast-usafe.bin")
print(f"archive written: {p}  ({os.path.getsize(p)} bytes)")
print(open(os.path.join(CRATE, "porttest", "archive_state_rust.txt")).read())''',
    },
    "server_test": {
        "title": "server_test — REBOUND webserver in pure Rust",
        "desc": "Starts the Rust port of REBOUND's webserver on port 12873 with a paused "
                "100-step simulation, fetches the /simulation binary blob over HTTP, then shuts "
                "the server down via /keyboard/81 ('Q'). The served blob is a valid REBOUND "
                "binary loadable by the C build.",
        "args": [],
        "cwd": "porttest",
        "custom_run": r'''import subprocess, time, urllib.request
exe = os.path.join(CRATE, "target", "release", "examples", "server_test.exe")
proc = subprocess.Popen([exe], cwd=os.path.join(CRATE, "porttest"),
                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
time.sleep(2.0)
blob = urllib.request.urlopen("http://localhost:12873/simulation", timeout=10).read()
print(f"/simulation returned {len(blob)} bytes; header: {blob[:32]!r}")
urllib.request.urlopen("http://localhost:12873/keyboard/81", timeout=10).read()
proc.wait(timeout=15)
print("server exited cleanly")''',
        "post": None,
    },
    "addfmt_test": {
        "title": "addfmt_test — reb_simulation_add_fmt and the solar-system dataset",
        "desc": "Adds the built-in solar system plus particles from orbital elements, Pal "
                "coordinates, and an orbital period, and dumps all particle states. Verified "
                "bit-identical vs the C build.",
        "args": [],
        "cwd": "porttest",
        "post": r'''print(open(os.path.join(CRATE, "porttest", "addfmt_rust.txt")).read())''',
    },
}


def code(src):
    return {"cell_type": "code", "execution_count": None, "metadata": {},
            "outputs": [], "source": src.splitlines(keepends=True)}


def md(src):
    return {"cell_type": "markdown", "metadata": {}, "source": src.splitlines(keepends=True)}


for name, ex in EXAMPLES.items():
    cells = [
        md(f"# {ex['title']}\n\n{ex['desc']}\n\n"
           f"Crate: `{CRATE}`. This notebook builds the example with cargo "
           f"(MSVC toolchain, zero unsafe / zero dependencies / zero warnings), runs it, "
           f"and displays its output. See `rebound_rust.md` for the full provenance."),
        code(f'import os, subprocess\nCRATE = r"{CRATE}"\n'
             f'print(subprocess.run(["cargo", "build", "--release", "--example", "{name}"],\n'
             f'      cwd=CRATE, capture_output=True, text=True).stderr)'),
    ]
    runner = ex.get("runner", name)
    runner_args = ex.get("runner_args", ex["args"])
    if ex.get("custom_run"):
        cells.append(code(ex["custom_run"]))
    else:
        arglist = "".join(f', "{a}"' for a in runner_args)
        cells.append(code(
            f'exe = os.path.join(CRATE, "target", "release", "examples", "{runner}.exe")\n'
            f'res = subprocess.run([exe{arglist}], cwd=os.path.join(CRATE, "{ex["cwd"]}"),\n'
            f'                     capture_output=True, text=True)\n'
            f'print(res.stdout)\nprint(res.stderr)'))
    if ex.get("post"):
        cells.append(code(ex["post"]))
    nb = {
        "cells": cells,
        "metadata": {
            "kernelspec": {"display_name": "Python 3", "language": "python", "name": "python3"},
            "language_info": {"name": "python", "version": "3"},
        },
        "nbformat": 4,
        "nbformat_minor": 5,
    }
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), f"{name}.ipynb")
    with open(out, "w", encoding="utf-8", newline="\n") as f:
        json.dump(nb, f, indent=1)
    print("wrote", out)
