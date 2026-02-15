# LeetCode 2390: Removing Stars From A String
# Python version

def removeStars(s) :
    res = []
    for c in s :
        if res and c == '*':
            res.pop()
        else:
            res.append(c)
    return ''.join(res)



def main():
    print(removeStars("leet**cod*e"))

if __name__ == "__main__":
    main()
