"""Build an REA import-managed-reconstruction payload from ILSpy output.

Usage: python3 -I rea_reconstruction.py <members.json> <ilspy project dir> <out.json>
Methods are located by declaring type file + name; overloaded names are matched by parameter count.
"""
import json
import os
import re
import sys


def method_spans(text):
    """Yield (name, param_count, start_line, end_line) for each method/ctor body in a C# file."""
    lines = text.split("\n")
    sig = re.compile(r"^\t(?:public|private|protected|internal)[^=;(]*?\b(\w+)\s*\(([^)]*)\)\s*(?::\s*base\(.*\))?$")
    i = 0
    while i < len(lines):
        m = sig.match(lines[i])
        if m and i + 1 < len(lines) and lines[i + 1].strip() == "{":
            depth, j = 0, i + 1
            while True:
                depth += lines[j].count("{") - lines[j].count("}")
                if depth == 0:
                    break
                j += 1
            params = [p for p in m.group(2).split(",") if p.strip()]
            yield m.group(1), len(params), i + 1, j + 1
            i = j
        i += 1


def main(members_path, project, out_path):
    evidence = json.load(open(members_path))
    result = evidence["normalized_result"]
    methods = []
    skipped = []
    for method in result["methods"]:
        body = method["body"]
        if body["status"] != "complete" and body["il_size"] == 0:
            continue
        declaring = method["declaring_type"]
        ns, _, cls = declaring.rpartition(".")
        path = os.path.join(project, ns or "", cls + ".cs")
        if not os.path.exists(path):
            skipped.append((declaring, method["name"], "no source file"))
            continue
        text = open(path, encoding="utf-8").read()
        name = cls if method["name"] in (".ctor",) else method["name"]
        candidates = [s for s in method_spans(text)
                      if s[0] == name and s[1] == method["signature"]["parameter_count"]]
        if len(candidates) != 1:
            skipped.append((declaring, method["name"], f"{len(candidates)} source candidates"))
            continue
        _, _, start, end = candidates[0]
        src_lines = text.split("\n")[start - 1:end]
        methods.append({
            "token": method["token"],
            "signature_sha256": method["signature"]["raw_sha256"],
            "normalized_il_sha256": body["normalized_il_sha256"],
            "reconstruction": {
                "kind": "decompiled-csharp",
                "language": "csharp",
                "text": "\n".join(src_lines),
                "source_path": os.path.abspath(path),
                "start_line": start,
                "end_line": end,
            },
        })
    payload = {
        "static_members": evidence,
        "decompiler": {
            "name": "ilspycmd",
            "version": "9.1.0.7988",
            "family": "ilspy",
            "executable_sha256": None,
            "options": ["-p", "-r", "xna31/ref", "-o", "cs2"],
        },
        "methods": methods,
        "notes": ["References resolved against Microsoft XNA Framework Redistributable 3.1 assemblies."],
    }
    json.dump(payload, open(out_path, "w"))
    print(f"{len(methods)} methods mapped, {len(skipped)} skipped")
    for s in skipped:
        print("  skipped:", *s)


if __name__ == "__main__":
    main(*sys.argv[1:4])
