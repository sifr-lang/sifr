
# LeetCode 1980: Find Unique Binary String
# Python version

def findDifferentBinaryString(nums: list[str]) -> str:
    out = []
    for i, row in enumerate(nums):
        out.append("1" if row[i] == "0" else "0")
    return "".join(out)
    



def main():
    nums1 = ['01', '10']
    ans1 = findDifferentBinaryString(nums1)
    assert len(ans1) == len(nums1)
    assert ans1 not in nums1

    nums2 = ['00', '01']
    ans2 = findDifferentBinaryString(nums2)
    assert len(ans2) == len(nums2)
    assert ans2 not in nums2

    nums3 = ['111', '011', '001']
    ans3 = findDifferentBinaryString(nums3)
    assert len(ans3) == len(nums3)
    assert ans3 not in nums3

if __name__ == "__main__":
    main()
