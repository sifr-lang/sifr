# LeetCode 278: First Bad Version
# Python version

def firstBadVersion(n: int) -> int:
    l, r = 1, n
    while l < r:
        v = (l + r) // 2
        if isBadVersion(v):
            r = v
        else:
            l = v + 1
    return l



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
