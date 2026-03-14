# LeetCode 881: Boats To Save People
# Python version

def numRescueBoats(people: list[int], limit: int) -> int:
    people.sort()
    right = len(people) - 1
    left = res = 0
    while left <= right:
        if people[left] + people[right] <= limit:
            left += 1
        right -= 1
        res += 1
    return res



def main():
    assert numRescueBoats([1,2], 3) == 1
    assert numRescueBoats([3,2,2,1], 3) == 3

if __name__ == "__main__":
    main()
