# LeetCode 349: Intersection Of Two Arrays
# Python version

def intersection(nums1: List[int], nums2: List[int]) -> List[int]:
    seen = set(nums1)

    res = []
    for n in nums2:
        if n in seen:
            res.append(n)
            seen.remove(n)
    return res        



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
