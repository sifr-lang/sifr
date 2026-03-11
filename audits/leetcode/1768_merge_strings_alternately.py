# LeetCode 1768: Merge Strings Alternately
# Python version with test cases

def mergeAlternately(word1: str, word2: str) -> str:
    i: int = 0
    j: int = 0
    result: str = ""
    while i < len(word1) and j < len(word2):
        result = result + word1[i]
        result = result + word2[j]
        i += 1
        j += 1
    while i < len(word1):
        result = result + word1[i]
        i += 1
    while j < len(word2):
        result = result + word2[j]
        j += 1
    return result

def main():
    print(mergeAlternately("abc", "pqr"))
    print(mergeAlternately("ab", "pqrs"))
    print(mergeAlternately("abcd", "pq"))

if __name__ == "__main__":
    main()
