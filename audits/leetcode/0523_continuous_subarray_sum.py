# LeetCode 523: Continuous Subarray Sum
# Python version

def checkSubarraySum(nums: List[int], k: int) -> bool:
    hashmap = {}
    hashmap[0]=-1
    summ=0
    for i,j in enumerate(nums):
        summ+=j
        if summ%k in hashmap.keys():
            if i-hashmap[summ%k]>=2:
                return True
            else:
                continue
        hashmap[summ%k]=i
    return False
        



def main():
    assert checkSubarraySum([23,2,4,6,7], 6) == True

if __name__ == "__main__":
    main()
