# LeetCode 1461: Check If A String Contains All Binary Codes Of Size K
# Python version

def hasAllCodes(s: str, k: int) -> bool:
    return len(set(s[i : i + k] for i in range(len(s) - k + 1))) == 2**k



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
