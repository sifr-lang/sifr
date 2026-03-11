# LeetCode 1209: Remove All Adjacent Duplicates In String Ii
# Python version

def removeDuplicates(s: str, k: int) -> str:
    stack = []  # [char, count]

    for c in s:
        if stack and stack[-1][0] == c:
            stack[-1][1] += 1
        else:
            stack.append([c, 1])

        if stack[-1][1] == k:
            stack.pop()

    res = ""
    for char, count in stack:
        res += char * count

    return res



def main():
    print(removeDuplicates("abcd", 2))
    print(removeDuplicates("deeedbbcccbdaa", 3))

if __name__ == "__main__":
    main()
