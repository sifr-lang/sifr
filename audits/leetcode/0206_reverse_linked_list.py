# LeetCode 206: Reverse Linked List
# Python version

def reverseList(head: ListNode) -> ListNode:
    prev, curr = None, head

    while curr:
        temp = curr.next
        curr.next = prev
        prev = curr
        curr = temp
    return prev



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
