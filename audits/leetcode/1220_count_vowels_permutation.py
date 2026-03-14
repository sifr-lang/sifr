# LeetCode 1220: Count Vowels Permutation
# Python version

Memo = {}    

def countVowelPermutation(n, c = '') -> int:        
    if (c, n) in Memo:            
        return Memo[(c, n)]
    if n == 1:
        if c == 'a':
            return 1 
        if c == 'e':
            return 2 
        if c == 'i':
            return 4 
        if c == 'o':
            return 2 
        if c == 'u':
            return 1            
        if c == '':                
            return 5
    else:
        if c == 'a':
            Memo[('a', n)] = countVowelPermutation(n - 1, 'e')                
            return Memo[('a', n)]
        if c == 'e':
            Memo[('e', n)] = countVowelPermutation(n - 1, 'a') + countVowelPermutation(n - 1, 'i')                
            return Memo[('e', n)]
        if c == 'i':
            Memo[('i', n)] = countVowelPermutation(n - 1, 'a') + countVowelPermutation(n - 1, 'e') + countVowelPermutation(n - 1, 'o') + countVowelPermutation(n - 1, 'u')          
            return Memo[('i', n)]
        if c == 'o':
            Memo[('o', n)] = countVowelPermutation(n - 1, 'i') + countVowelPermutation(n - 1, 'u')                
            return Memo[('o', n)]
        if c == 'u':
            Memo[('u', n)] = countVowelPermutation(n - 1, 'a')                
            return Memo[('u', n)]
        if c == '':
            Tot = 0
            for i in ['a', 'e', 'i', 'o', 'u']:
                Tot = Tot + countVowelPermutation(n - 1, i);                    
            return Tot % 1000000007         



def main():
    assert countVowelPermutation(1) == 5
    assert countVowelPermutation(2) == 10
    assert countVowelPermutation(5) == 68

if __name__ == "__main__":
    main()
