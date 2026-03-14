#!/usr/bin/env python3
"""
Batch converter: reads every LeetCode Python solution, converts to Sifr,
writes .py + .sifr files, and records which ones need test cases.

This script handles the CONVERSION only. Testing is done separately.
"""
import os, re

SRC = "/Users/yaseralnajjar/work/sifr/leetcode/python"
DST = "/Users/yaseralnajjar/work/sifr/codebase/audit/leetcode"

DONE = {
    "0009","0014","0045","0053","0055","0058","0070","0121",
    "0134","0152","0169","0198","0238","0300","0392","0509",
    "0605","0704","1768","1929",
}

def slug(fn):
    return fn.replace(".py","").replace("-","_")

def num(fn):
    return fn[:4]

def nice_name(fn):
    n = fn.replace(".py","")
    n = re.sub(r'^\d+-','',n)
    return n.replace("-"," ").title()

def read(path):
    with open(path) as f:
        return f.read()

def dedent_method(lines, base_indent):
    """Remove base_indent spaces from each line."""
    out = []
    for l in lines:
        if l.strip() == "":
            out.append("")
        elif l.startswith(" " * base_indent):
            out.append(l[base_indent:])
        elif l.startswith("\t" * (base_indent // 4 or 1)):
            out.append(l[(base_indent // 4 or 1):])
        else:
            out.append(l)
    return out

def extract_solution_body(src):
    """Extract the body of a Solution class, converting methods to functions."""
    lines = src.split("\n")
    
    # Find all imports
    imports = []
    for l in lines:
        stripped = l.strip()
        if stripped.startswith("import ") or stripped.startswith("from "):
            imports.append(stripped)
    
    # Find class Solution
    in_class = False
    class_indent = 0
    class_body = []
    
    for l in lines:
        if re.match(r'^class\s+Solution', l):
            in_class = True
            class_indent = 4  # standard
            continue
        if in_class:
            # Check if we've left the class
            if l.strip() and not l.startswith(" ") and not l.startswith("\t") and not l.strip().startswith("#"):
                break
            class_body.append(l)
    
    if not class_body:
        # No Solution class - return source as-is (might be standalone)
        return src, imports
    
    # Convert class body to standalone functions
    result_lines = []
    for l in class_body:
        # Remove one level of indentation
        if l.startswith("    "):
            l = l[4:]
        elif l.startswith("\t"):
            l = l[1:]
        
        # Remove self parameter
        l = re.sub(r'\(self,\s*', '(', l)
        l = re.sub(r'\(self\)', '()', l)
        
        # Remove self. references
        l = l.replace("self.", "")
        
        result_lines.append(l)
    
    return "\n".join(result_lines), imports

def python_to_sifr(func_text, imports):
    """Convert Python function text to Sifr syntax."""
    t = func_text
    
    # Type hint conversions
    t = re.sub(r'List\[List\[List\[(\w+)\]\]\]', r'list[list[list[\1]]]', t)
    t = re.sub(r'List\[List\[(\w+)\]\]', r'list[list[\1]]', t)
    t = re.sub(r'List\[(\w+)\]', r'list[\1]', t)
    t = re.sub(r'Optional\[(\w+)\]', r'\1 | None', t)
    t = re.sub(r'Dict\[(\w+),\s*(\w+)\]', r'dict[\1, \2]', t)
    t = re.sub(r'Set\[(\w+)\]', r'set[\1]', t)
    t = re.sub(r'Tuple\[([^\]]+)\]', r'tuple[\1]', t)
    
    # Remove docstrings
    t = re.sub(r'"""[\s\S]*?"""', '', t)
    t = re.sub(r"'''[\s\S]*?'''", '', t)
    
    # Remove type: ignore comments
    t = re.sub(r'#\s*type:\s*ignore.*', '', t)
    
    # Remove blank lines at start
    lines = t.split("\n")
    while lines and lines[0].strip() == "":
        lines.pop(0)
    
    return "\n".join(lines)

def extract_func_name(sifr_text):
    """Extract the first function name from Sifr text."""
    m = re.search(r'def\s+(\w+)\s*\(', sifr_text)
    if m:
        return m.group(1)
    return None

def write_file(path, content):
    with open(path, "w") as f:
        f.write(content)

def main():
    files = sorted(os.listdir(SRC))
    
    converted = 0
    skipped = 0
    
    manifest = []  # (slug, func_name, has_tests, category)
    
    for fn in files:
        if not fn.endswith(".py"):
            continue
        n = num(fn)
        if n in DONE:
            skipped += 1
            continue
        
        s = slug(fn)
        name = nice_name(fn)
        src = read(os.path.join(SRC, fn))
        
        # Categorize
        has_node = bool(re.search(r'ListNode|TreeNode|Optional\[', src))
        has_init = bool(re.search(r'__init__', src))
        
        if has_node and not has_init:
            cat = "node"
        elif has_init:
            cat = "design"
        else:
            cat = "algo"
        
        # Extract and convert
        body, imports = extract_solution_body(src)
        sifr_body = python_to_sifr(body, imports)
        func_name = extract_func_name(sifr_body)
        
        if func_name is None:
            # Can't find a function - skip
            manifest.append((s, "???", False, cat, name))
            continue
        
        # Write Sifr file (without main/test cases - those will be added by test runner)
        sifr_header = f"# LeetCode {int(n)}: {name}\n\n"
        sifr_content = sifr_header + sifr_body.rstrip() + "\n\n"
        sifr_content += "def main():\n"
        sifr_content += f'    print("TODO: add test cases")\n'
        
        # Write Python file
        py_header = f"# LeetCode {int(n)}: {name}\n# Python version with test cases\n\n"
        # Add necessary imports
        import_lines = ""
        for imp in imports:
            if "typing" not in imp and "collections" not in imp:
                import_lines += imp + "\n"
        
        py_content = py_header + import_lines + body.rstrip() + "\n\n"
        py_content += "def main():\n"
        py_content += f'    print("TODO: add test cases")\n\n'
        py_content += 'if __name__ == "__main__":\n    main()\n'
        
        write_file(os.path.join(DST, f"{s}.sifr"), sifr_content)
        write_file(os.path.join(DST, f"{s}.py"), py_content)
        
        manifest.append((s, func_name, False, cat, name))
        converted += 1
    
    # Write manifest
    with open(os.path.join(DST, "manifest.txt"), "w") as f:
        f.write("# slug|func_name|has_tests|category|name\n")
        for s, fn, ht, cat, name in manifest:
            f.write(f"{s}|{fn}|{ht}|{cat}|{name}\n")
    
    print(f"Converted: {converted}")
    print(f"Skipped (already done): {skipped}")
    print(f"Manifest written to {DST}/manifest.txt")

if __name__ == "__main__":
    main()
