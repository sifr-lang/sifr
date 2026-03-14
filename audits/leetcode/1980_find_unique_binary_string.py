# LeetCode 1980: Find Unique Binary String
# Python version

def findDifferentBinaryString(nums: List[str]) -> str:
    
    strSet = { s for s in nums }
    
    def backtrack(i, cur):
        if i == len(nums):
            res = "".join(cur)
            return None if res in strSet else res
        
        res = backtrack(i+1, cur)
        if res: return res
        
        cur[i] = "1"
        res = backtrack(i+1, cur)
        if res: return res
        
    return backtrack(0, ["0" for s in nums])
    



def main():
    assert findDifferentBinaryString(['01', '10']) == '00'
    assert findDifferentBinaryString(['00', '01']) == '11'
    assert findDifferentBinaryString(['111', '011', '001']) == '000'

if __name__ == "__main__":
    main()
