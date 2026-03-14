
# LeetCode 349: Intersection Of Two Arrays
# Python version

def intersection(nums1: list[int], nums2: list[int]) -> list[int]:
    seen = set(nums1)

    res = []
    for n in nums2:
        if n in seen:
            res.append(n)
            seen.remove(n)
    return res        



def main():
    assert intersection([1, 2, 2, 1], [2, 2]) == [2]
    assert intersection([4, 9, 5], [9, 4, 9, 8, 4]) == [9, 4]

if __name__ == "__main__":
    main()
