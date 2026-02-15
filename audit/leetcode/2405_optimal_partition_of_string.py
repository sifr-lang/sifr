# LeetCode 2405: Optimal Partition Of String
# Python version

def partitionString(s: str) -> int:
    c=0
    res=set()
    for i in s:
        if i in res:
            c=c+1
            res=set()
        res.add(i)
    return c+1



def main():
    print(minPartitions("abacbc"))

if __name__ == "__main__":
    main()
