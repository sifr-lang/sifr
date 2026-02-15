# LeetCode 344: Reverse String
# Python version

def reverseString(s: List[str]) -> None:
    """
    Do not return anything, modify s in-place instead.
    """
    l = 0
    r = len(s) - 1
    while l < r:
        s[l],s[r] = s[r],s[l]
        l += 1
        r -= 1



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
