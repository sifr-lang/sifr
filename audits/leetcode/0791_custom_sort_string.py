# LeetCode 791: Custom Sort String
# Python version

def customSortString(order: str, s: str) -> str:
    char_count_of_s = {}
    for i in s:
        char_count_of_s[i] = char_count_of_s.get(i, 0) + 1
    
    satisfied_string = ""
    for char in order:
        if char in char_count_of_s:
            satisfied_string += char * char_count_of_s[char]
            del char_count_of_s[char]
    
    for key,val in char_count_of_s.items():
        satisfied_string += key * val

    return satisfied_string



def main():
    assert customSortString("cba", "abcd") == 'cbad'

if __name__ == "__main__":
    main()
