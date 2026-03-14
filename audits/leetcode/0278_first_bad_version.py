
# LeetCode 278: First Bad Version
# Python version

BAD_VERSION = 4

def isBadVersion(version: int) -> bool:
    return version >= BAD_VERSION

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
    assert firstBadVersion(5) == 4
    assert firstBadVersion(4) == 4

if __name__ == "__main__":
    main()
