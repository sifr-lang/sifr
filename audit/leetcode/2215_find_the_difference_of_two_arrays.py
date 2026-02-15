# LeetCode 2215: Find The Difference Of Two Arrays
# Python version

def findDifference(nums1: List[int], nums2: List[int]) -> List[List[int]]:
    nums1 = set(nums1)
    nums2 = set(nums2)
    table = {}
    for _, val in enumerate(nums2):
        table[val] = 1

    unik1 = []
    unik2 = []
    for i in nums1:
        if i in table:
            table[i] += 1
        else:
            unik1.append(i)
    
    for key, val in table.items():
        if val == 1:
            unik2.append(key)
    return [unik1, unik2]

  
# Time Complexity: O(m + n), we check each element of nums1Set and nums2Set
# Space Complexity: O(m + n), where m and n are length sets in worst case.




def findDifference(nums1: List[int], nums2: List[int]) -> List[List[int]]:
    nums1_set = set(nums1)
    nums2_set = set(nums2)

    lst1 = [num for num in nums1_set if num not in nums2_set]
    lst2 = [num for num in nums2_set if num not in nums1_set]

    return [lst1, lst2]




def main():
    print("no test cases")

if __name__ == "__main__":
    main()
