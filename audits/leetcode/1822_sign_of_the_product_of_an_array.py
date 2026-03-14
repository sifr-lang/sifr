# LeetCode 1822: Sign Of The Product Of An Array
# Python version

def arraySign(nums: List[int]) -> int:
    flag = True
    for i in nums:
        if i == 0:
            return 0
        if i < 0:
            flag = not flag
    
    return 1 if flag else -1



def main():
    assert arraySign([-1,-2,-3,-4,3,2,1]) == 1
    assert arraySign([1,5,0,2,-3]) == 0
    assert arraySign([-1,1,-1,1,-1]) == -1

if __name__ == "__main__":
    main()
