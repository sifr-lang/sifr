# LeetCode 876: Middle Of The Linked List
# Python version

def middleNode(head: Optional[ListNode]) -> Optional[ListNode]:
    if not head or not head.next:
        return head

    slow = fast = head
    while fast and fast.next:
        slow, fast = slow.next, fast.next.next

    return slow



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
