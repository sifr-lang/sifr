# LeetCode 71: Simplify Path
# Python version

def simplifyPath(path: str) -> str:

    stack = []

    for i in path.split("/"):
        #  if i == "/" or i == '//', it becomes '' (empty string)

        if i == "..":
            if stack:
                stack.pop()
        elif i == "." or i == '':
            # skip "." or an empty string
            continue
        else:
            stack.append(i)

    res = "/" + "/".join(stack)
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
