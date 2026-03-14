# LeetCode 13: Roman To Integer
# Python version

def romanToInt(s: str) -> int:
    roman = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100, "D": 500, "M": 1000}
    res = 0
    for i in range(len(s)):
        if i + 1 < len(s) and roman[s[i]] < roman[s[i + 1]]:
            res -= roman[s[i]]
        else:
            res += roman[s[i]]
    return res



def main():
    assert romanToInt("III") == 3
    assert romanToInt("LVIII") == 58
    assert romanToInt("MCMXCIV") == 1994

if __name__ == "__main__":
    main()
