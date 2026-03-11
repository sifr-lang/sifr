# LeetCode 147: Insertion Sort List
# Python version

def insertionSortList(head: Optional[ListNode]) -> Optional[ListNode]:
    if not head or not head.next:
        return head

    sentinel = ListNode()
    curr = head
    while curr:
        prev = sentinel
        while prev.next and curr.val >= prev.next.val:
            prev = prev.next

        curr.next, prev.next, curr = prev.next, curr, curr.next

    return sentinel.next



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
