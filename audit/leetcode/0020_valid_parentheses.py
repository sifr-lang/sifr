# LeetCode 20: Valid Parentheses
# Python version

def isValid(s: str) -> bool:
    bracketMap = {")": "(", "]": "[", "}": "{"}
    stack = []

    for c in s:
        if c not in bracketMap:
            stack.append(c)
            continue
        if not stack or stack[-1] != bracketMap[c]:
            return False
        stack.pop()

    return not stack



def main():
    print(isValid("()"))
    print(isValid("()[]{}"))
    print(isValid("(]"))

if __name__ == "__main__":
    main()
