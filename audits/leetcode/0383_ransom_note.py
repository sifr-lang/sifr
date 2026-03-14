# LeetCode 383: Ransom Note
# Python version

def canConstruct(ransomNote: str, magazine: str) -> bool:
    r_counter = Counter(ransomNote)
    m_counter = Counter(magazine)
    # magazine contains (>=) ransomNote
    for c in ransomNote:
        if m_counter[c] < r_counter[c]:
            return False
    return True



def main():
    assert canConstruct("a", "b") == False
    assert canConstruct("aa", "aab") == True

if __name__ == "__main__":
    main()
