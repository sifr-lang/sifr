# LeetCode 160: Intersection Of Two Linked Lists
# Python version

def getIntersectionNode(
    self, headA: ListNode, headB: ListNode
) -> Optional[ListNode]:
    l1, l2 = headA, headB
    while l1 != l2:
        l1 = l1.next if l1 else headB
        l2 = l2.next if l2 else headA
    return l1



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
