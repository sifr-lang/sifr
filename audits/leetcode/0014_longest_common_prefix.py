# LeetCode 14: Longest Common Prefix
# Python version with test cases

def longestCommonPrefix(strs: list[str]) -> str:
    if len(strs) == 0:
        return ""
    prefix: str = strs[0]
    for i in range(1, len(strs)):
        while not strs[i].startswith(prefix):
            prefix = prefix[:len(prefix) - 1]
            if len(prefix) == 0:
                return ""
    return prefix

def main():
    print(longestCommonPrefix(["flower", "flow", "flight"]))
    print(longestCommonPrefix(["dog", "racecar", "car"]))
    print(longestCommonPrefix(["interspecies", "interstellar", "interstate"]))
    print(longestCommonPrefix(["a"]))

if __name__ == "__main__":
    main()
