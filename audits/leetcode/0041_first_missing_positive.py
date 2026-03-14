# LeetCode 41: First Missing Positive
# Python version

def firstMissingPositive(nums: List[int]) -> int:
    A = nums
    for i in range(len(A)):
        if A[i] < 0:
            A[i] = 0
        
    for i in range(len(A)):
        val = abs(A[i])
        if 1 <= val <= len(A):
            if A[val - 1] > 0:
                A[val - 1] *= -1
            elif A[val - 1] == 0:
                A[val - 1] = -1 * (len(A) + 1)
    
    for i in range( 1, len(A)+ 1):
        if A[i -1] >= 0:
            return i
    
    return len(A) + 1
    

def firstMissingPositive_2(nums: List[int]) -> int:
    new = set(nums)
    i = 1
    while i in new:
        i += 1
    return i



def main():
    assert firstMissingPositive([1,2,0]) == 3
    assert firstMissingPositive([3,4,-1,1]) == 2
    assert firstMissingPositive([7,8,9,11,12]) == 1

if __name__ == "__main__":
    main()
