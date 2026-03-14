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
    arg0 = ['h', 'e', 'l', 'l', 'o']
    _result = reverseString(arg0)
    assert arg0 == ['o', 'l', 'l', 'e', 'h']
    arg0 = ['H', 'a', 'n', 'n', 'a', 'h']
    _result = reverseString(arg0)
    assert arg0 == ['h', 'a', 'n', 'n', 'a', 'H']

if __name__ == "__main__":
    main()
