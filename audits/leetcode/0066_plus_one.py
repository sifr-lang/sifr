# LeetCode 66: Plus One
# Python version

def plusOne(digits: List[int]) -> List[int]:
    one = 1
    i = 0
    digits = digits[::-1]

    while one:
        if i < len(digits):
            if digits[i] == 9:
                digits[i] = 0
            else:
                digits[i] += 1
                one = 0
        else:
            digits.append(one)
            one = 0
        i += 1
    return digits[::-1]



def main():
    print(plusOne([1,2,3]))
    print(plusOne([9]))

if __name__ == "__main__":
    main()
