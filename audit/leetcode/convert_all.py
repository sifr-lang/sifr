#!/usr/bin/env python3
"""
Automated LeetCode Python → Sifr converter.

For each Python LeetCode solution:
1. Reads the Python source
2. Extracts the solution method(s)
3. Converts to standalone Sifr function(s) with type annotations
4. Generates test cases
5. Writes both .py (with test harness) and .sifr files
"""

import os
import re
import sys
import textwrap

PYTHON_DIR = "/Users/yaseralnajjar/work/sifr/leetcode/python"
OUTPUT_DIR = "/Users/yaseralnajjar/work/sifr/codebase/audit/leetcode"

# Already done problems (skip these)
ALREADY_DONE = {
    "0009", "0014", "0045", "0053", "0055", "0058", "0070", "0121",
    "0134", "0152", "0169", "0198", "0238", "0300", "0392", "0509",
    "0605", "0704", "1768", "1929"
}

# ─── Test case database ───────────────────────────────────────────────
# Maps problem number to (function_name, list of (args_str_py, args_str_sifr, expected_py, expected_sifr))
# args_str is the arguments as they'd appear in a function call
# If args_str_sifr is None, it's the same as args_str_py

def make_test_cases():
    """Generate test cases for all 376 remaining problems."""
    # We'll define test cases for each problem. For problems that use
    # data structures Sifr can't handle (ListNode, TreeNode, etc.),
    # we'll still create the files but mark them as requiring those structures.
    
    cases = {}
    
    # Helper to add a problem
    def add(num, func, tests):
        """tests is list of (args, expected) where args is a string and expected is a string"""
        cases[num] = (func, tests)
    
    # ─── Easy/Medium problems that work with basic types ───
    add("0001", "twoSum", [
        ("([2, 7, 11, 15], 9)", "[0, 1]"),
        ("([3, 2, 4], 6)", "[1, 2]"),
        ("([3, 3], 6)", "[0, 1]"),
    ])
    add("0002", "addTwoNumbers", [])  # ListNode
    add("0003", "lengthOfLongestSubstring", [
        ('("abcabcbb",)', "3"),
        ('("bbbbb",)', "1"),
        ('("pwwkew",)', "3"),
        ('("",)', "0"),
    ])
    add("0004", "findMedianSortedArrays", [
        ("([1, 3], [2])", "2.0"),
        ("([1, 2], [3, 4])", "2.5"),
    ])
    add("0005", "longestPalindrome", [
        ('("babad",)', "bab"),
        ('("cbbd",)', "bb"),
    ])
    add("0006", "convert", [
        ('("PAYPALISHIRING", 3)', "PAHNAPLSIIGYIR"),
        ('("PAYPALISHIRING", 4)', "PINALSIGYAHRPI"),
        ('("A", 1)', "A"),
    ])
    add("0007", "reverse", [
        ("(123,)", "321"),
        ("(-123,)", "-321"),
        ("(120,)", "21"),
        ("(0,)", "0"),
    ])
    add("0010", "isMatch", [
        ('("aa", "a")', "False"),
        ('("aa", "a*")', "True"),
        ('("ab", ".*")', "True"),
    ])
    add("0011", "maxArea", [
        ("([1, 8, 6, 2, 5, 4, 8, 3, 7],)", "49"),
        ("([1, 1],)", "1"),
    ])
    add("0012", "intToRoman", [
        ("(3749,)", "MMMDCCXLIX"),
        ("(58,)", "LVIII"),
        ("(1994,)", "MCMXCIV"),
    ])
    add("0013", "romanToInt", [
        ('("III",)', "3"),
        ('("LVIII",)', "58"),
        ('("MCMXCIV",)', "1994"),
    ])
    add("0015", "threeSum", [
        ("([-1, 0, 1, 2, -1, -4],)", "[[-1, -1, 2], [-1, 0, 1]]"),
        ("([0, 1, 1],)", "[]"),
        ("([0, 0, 0],)", "[[0, 0, 0]]"),
    ])
    add("0016", "threeSumClosest", [
        ("([-1, 2, 1, -4], 1)", "2"),
        ("([0, 0, 0], 1)", "0"),
    ])
    add("0017", "letterCombinations", [
        ('("23",)', '["ad", "ae", "af", "bd", "be", "bf", "cd", "ce", "cf"]'),
        ('("",)', "[]"),
        ('("2",)', '["a", "b", "c"]'),
    ])
    add("0018", "fourSum", [
        ("([1, 0, -1, 0, -2, 2], 0)", "[[-2, -1, 1, 2], [-2, 0, 0, 2], [-1, 0, 0, 1]]"),
    ])
    add("0019", "removeNthFromEnd", [])  # ListNode
    add("0020", "isValid", [
        ('("()",)', "True"),
        ('("()[]{}",)', "True"),
        ('("(]",)', "False"),
        ('("([)]",)', "False"),
    ])
    add("0021", "mergeTwoLists", [])  # ListNode
    add("0022", "generateParenthesis", [
        ("(3,)", '["((()))", "(()())", "(())()", "()(())", "()()()"]'),
        ("(1,)", '["()"]'),
    ])
    add("0023", "mergeKLists", [])  # ListNode
    add("0024", "swapPairs", [])  # ListNode
    add("0025", "reverseKGroup", [])  # ListNode
    add("0026", "removeDuplicates", [
        ("([1, 1, 2],)", "2"),
        ("([0, 0, 1, 1, 1, 2, 2, 3, 3, 4],)", "5"),
    ])
    add("0027", "removeElement", [
        ("([3, 2, 2, 3], 3)", "2"),
        ("([0, 1, 2, 2, 3, 0, 4, 2], 2)", "5"),
    ])
    add("0028", "strStr", [
        ('("sadbutsad", "sad")', "0"),
        ('("leetcode", "leeto")', "-1"),
        ('("hello", "ll")', "2"),
    ])
    add("0033", "search", [
        ("([4, 5, 6, 7, 0, 1, 2], 0)", "4"),
        ("([4, 5, 6, 7, 0, 1, 2], 3)", "-1"),
        ("([1], 0)", "-1"),
    ])
    add("0034", "searchRange", [
        ("([5, 7, 7, 8, 8, 10], 8)", "[3, 4]"),
        ("([5, 7, 7, 8, 8, 10], 6)", "[-1, -1]"),
        ("([], 0)", "[-1, -1]"),
    ])
    add("0035", "searchInsert", [
        ("([1, 3, 5, 6], 5)", "2"),
        ("([1, 3, 5, 6], 2)", "1"),
        ("([1, 3, 5, 6], 7)", "4"),
    ])
    add("0036", "isValidSudoku", [])  # Complex 2D array input
    add("0039", "combinationSum", [
        ("([2, 3, 6, 7], 7)", "[[2, 2, 3], [7]]"),
    ])
    add("0040", "combinationSum2", [
        ("([10, 1, 2, 7, 6, 1, 5], 8)", "[[1, 1, 6], [1, 2, 5], [1, 7], [2, 6]]"),
    ])
    add("0041", "firstMissingPositive", [
        ("([1, 2, 0],)", "3"),
        ("([3, 4, -1, 1],)", "2"),
        ("([7, 8, 9, 11, 12],)", "1"),
    ])
    add("0042", "trap", [
        ("([0, 1, 0, 2, 1, 0, 1, 3, 2, 1, 2, 1],)", "6"),
        ("([4, 2, 0, 3, 2, 5],)", "9"),
    ])
    add("0043", "multiply", [
        ('("2", "3")', "6"),
        ('("123", "456")', "56088"),
    ])
    add("0044", "isMatch", [
        ('("aa", "a")', "False"),
        ('("aa", "*")', "True"),
        ('("cb", "?a")', "False"),
    ])
    add("0046", "permute", [
        ("([1, 2, 3],)", "[[1, 2, 3], [1, 3, 2], [2, 1, 3], [2, 3, 1], [3, 1, 2], [3, 2, 1]]"),
    ])
    add("0047", "permuteUnique", [
        ("([1, 1, 2],)", "[[1, 1, 2], [1, 2, 1], [2, 1, 1]]"),
    ])
    add("0048", "rotate", [])  # In-place matrix rotation
    add("0049", "groupAnagrams", [])  # Complex output ordering
    add("0050", "myPow", [
        ("(2.0, 10)", "1024.0"),
        ("(2.1, 3)", "9.261"),
        ("(2.0, -2)", "0.25"),
    ])
    add("0051", "solveNQueens", [])  # Complex
    add("0052", "totalNQueens", [
        ("(4,)", "2"),
        ("(1,)", "1"),
    ])
    add("0054", "spiralOrder", [
        ("([[1, 2, 3], [4, 5, 6], [7, 8, 9]],)", "[1, 2, 3, 6, 9, 8, 7, 4, 5]"),
    ])
    add("0056", "merge", [
        ("([[1, 3], [2, 6], [8, 10], [15, 18]],)", "[[1, 6], [8, 10], [15, 18]]"),
    ])
    add("0057", "insert", [])  # Complex interval manipulation
    add("0061", "rotateRight", [])  # ListNode
    add("0062", "uniquePaths", [
        ("(3, 7)", "28"),
        ("(3, 2)", "3"),
    ])
    add("0063", "uniquePathsWithObstacles", [
        ("([[0, 0, 0], [0, 1, 0], [0, 0, 0]],)", "2"),
    ])
    add("0064", "minPathSum", [
        ("([[1, 3, 1], [1, 5, 1], [4, 2, 1]],)", "7"),
    ])
    add("0066", "plusOne", [
        ("([1, 2, 3],)", "[1, 2, 4]"),
        ("([4, 3, 2, 1],)", "[4, 3, 2, 2]"),
        ("([9],)", "[1, 0]"),
    ])
    add("0067", "addBinary", [
        ('("11", "1")', "100"),
        ('("1010", "1011")', "10101"),
    ])
    add("0068", "fullJustify", [])  # Complex string formatting
    add("0069", "mySqrt", [
        ("(4,)", "2"),
        ("(8,)", "2"),
        ("(0,)", "0"),
        ("(1,)", "1"),
    ])
    add("0071", "simplifyPath", [
        ('("/home/",)', "/home"),
        ('("/home//foo/",)', "/home/foo"),
        ('("/home/user/Documents/../Pictures",)', "/home/user/Pictures"),
        ('("/../",)', "/"),
    ])
    add("0072", "minDistance", [
        ('("horse", "ros")', "3"),
        ('("intention", "execution")', "5"),
    ])
    add("0073", "setZeroes", [])  # In-place matrix
    add("0074", "searchMatrix", [
        ("([[1, 3, 5, 7], [10, 11, 16, 20], [23, 30, 34, 60]], 3)", "True"),
        ("([[1, 3, 5, 7], [10, 11, 16, 20], [23, 30, 34, 60]], 13)", "False"),
    ])
    add("0075", "sortColors", [])  # In-place
    add("0076", "minWindow", [
        ('("ADOBECODEBANC", "ABC")', "BANC"),
        ('("a", "a")', "a"),
        ('("a", "aa")', ""),
    ])
    add("0077", "combine", [
        ("(4, 2)", "[[1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]"),
    ])
    add("0078", "subsets", [
        ("([1, 2, 3],)", "[[], [1], [1, 2], [1, 2, 3], [1, 3], [2], [2, 3], [3]]"),
    ])
    add("0079", "exist", [])  # 2D board
    add("0080", "removeDuplicates", [
        ("([1, 1, 1, 2, 2, 3],)", "5"),
    ])
    add("0081", "search", [
        ("([2, 5, 6, 0, 0, 1, 2], 0)", "True"),
        ("([2, 5, 6, 0, 0, 1, 2], 3)", "False"),
    ])
    add("0083", "deleteDuplicates", [])  # ListNode
    add("0084", "largestRectangleArea", [
        ("([2, 1, 5, 6, 2, 3],)", "10"),
        ("([2, 4],)", "4"),
    ])
    add("0086", "partition", [])  # ListNode
    add("0088", "merge", [])  # In-place
    add("0090", "subsetsWithDup", [
        ("([1, 2, 2],)", "[[], [1], [1, 2], [1, 2, 2], [2], [2, 2]]"),
    ])
    add("0091", "numDecodings", [
        ('("12",)', "2"),
        ('("226",)', "3"),
        ('("06",)', "0"),
    ])
    add("0092", "reverseBetween", [])  # ListNode
    add("0094", "inorderTraversal", [])  # TreeNode
    add("0097", "isInterleave", [
        ('("aabcc", "dbbca", "aadbbcbcac")', "True"),
        ('("aabcc", "dbbca", "aadbbbaccc")', "False"),
        ('("", "", "")', "True"),
    ])
    add("0098", "isValidBST", [])  # TreeNode
    add("0100", "isSameTree", [])  # TreeNode
    add("0101", "isSymmetric", [])  # TreeNode
    add("0102", "levelOrder", [])  # TreeNode
    add("0103", "zigzagLevelOrder", [])  # TreeNode
    add("0104", "maxDepth", [])  # TreeNode
    add("0105", "buildTree", [])  # TreeNode
    add("0106", "buildTree", [])  # TreeNode
    add("0108", "sortedArrayToBST", [])  # TreeNode
    add("0110", "isBalanced", [])  # TreeNode
    add("0112", "hasPathSum", [])  # TreeNode
    add("0115", "numDistinct", [
        ('("rabbbit", "rabbit")', "3"),
        ('("babgbag", "bag")', "5"),
    ])
    add("0118", "generate", [
        ("(5,)", "[[1], [1, 1], [1, 2, 1], [1, 3, 3, 1], [1, 4, 6, 4, 1]]"),
    ])
    add("0119", "getRow", [
        ("(3,)", "[1, 3, 3, 1]"),
        ("(0,)", "[1]"),
        ("(1,)", "[1, 1]"),
    ])
    add("0120", "minimumTotal", [
        ("([[2], [3, 4], [6, 5, 7], [4, 1, 8, 3]],)", "11"),
    ])
    add("0122", "maxProfit", [
        ("([7, 1, 5, 3, 6, 4],)", "7"),
        ("([1, 2, 3, 4, 5],)", "4"),
        ("([7, 6, 4, 3, 1],)", "0"),
    ])
    add("0124", "maxPathSum", [])  # TreeNode
    add("0125", "isPalindrome", [
        ('("A man, a plan, a canal: Panama",)', "True"),
        ('("race a car",)', "False"),
        ('(" ",)', "True"),
    ])
    add("0127", "ladderLength", [])  # Complex
    add("0128", "longestConsecutive", [
        ("([100, 4, 200, 1, 3, 2],)", "4"),
        ("([0, 3, 7, 2, 5, 8, 4, 6, 0, 1],)", "9"),
    ])
    add("0130", "solve", [])  # In-place 2D
    add("0131", "partition", [
        ('("aab",)', '[["a", "a", "b"], ["aa", "b"]]'),
    ])
    add("0133", "cloneGraph", [])  # Graph node
    add("0135", "candy", [
        ("([1, 0, 2],)", "5"),
        ("([1, 2, 2],)", "4"),
    ])
    add("0136", "singleNumber", [
        ("([2, 2, 1],)", "1"),
        ("([4, 1, 2, 1, 2],)", "4"),
    ])
    add("0138", "copyRandomList", [])  # Special node
    add("0139", "wordBreak", [
        ('("leetcode", ["leet", "code"])', "True"),
        ('("applepenapple", ["apple", "pen"])', "True"),
        ('("catsandog", ["cats", "dog", "sand", "and", "cat"])', "False"),
    ])
    add("0141", "hasCycle", [])  # ListNode
    add("0143", "reorderList", [])  # ListNode
    add("0144", "preorderTraversal", [])  # TreeNode
    add("0145", "postorderTraversal", [])  # TreeNode
    add("0146", "LRUCache", [])  # Class design
    add("0147", "insertionSortList", [])  # ListNode
    add("0148", "sortList", [])  # ListNode
    add("0149", "maxPoints", [])  # Complex
    add("0150", "evalRPN", [
        ('(["2", "1", "+", "3", "*"],)', "9"),
        ('(["4", "13", "5", "/", "+"],)', "6"),
    ])
    add("0151", "reverseWords", [
        ('("the sky is blue",)', "blue is sky the"),
        ('("  hello world  ",)', "world hello"),
    ])
    add("0153", "findMin", [
        ("([3, 4, 5, 1, 2],)", "1"),
        ("([4, 5, 6, 7, 0, 1, 2],)", "0"),
        ("([11, 13, 15, 17],)", "11"),
    ])
    add("0155", "MinStack", [])  # Class design
    add("0160", "getIntersectionNode", [])  # ListNode
    add("0162", "findPeakElement", [
        ("([1, 2, 3, 1],)", "2"),
        ("([1, 2, 1, 3, 5, 6, 4],)", "5"),
    ])
    add("0167", "twoSum", [
        ("([2, 7, 11, 15], 9)", "[1, 2]"),
        ("([2, 3, 4], 6)", "[1, 3]"),
        ("([-1, 0], -1)", "[1, 2]"),
    ])
    add("0168", "convertToTitle", [
        ("(1,)", "A"),
        ("(28,)", "AB"),
        ("(701,)", "ZY"),
    ])
    add("0179", "largestNumber", [
        ("([10, 2],)", "210"),
        ("([3, 30, 34, 5, 9],)", "9534330"),
    ])
    add("0187", "findRepeatedDnaSequences", [
        ('("AAAAACCCCCAAAAACCCCCCAAAAAGGGTTT",)', '["AAAAACCCCC", "CCCCCAAAAA"]'),
    ])
    add("0189", "rotate", [])  # In-place
    add("0190", "reverseBits", [])  # Bitwise
    add("0191", "hammingWeight", [])  # Bitwise
    add("0199", "rightSideView", [])  # TreeNode
    add("0200", "numIslands", [])  # 2D grid in-place
    add("0201", "rangeBitwiseAnd", [])  # Bitwise
    add("0202", "isHappy", [
        ("(19,)", "True"),
        ("(2,)", "False"),
    ])
    add("0203", "removeElements", [])  # ListNode
    add("0205", "isIsomorphic", [
        ('("egg", "add")', "True"),
        ('("foo", "bar")', "False"),
        ('("paper", "title")', "True"),
    ])
    add("0206", "reverseList", [])  # ListNode
    add("0207", "canFinish", [])  # Complex graph
    add("0208", "Trie", [])  # Class design
    add("0209", "minSubArrayLen", [
        ("(7, [2, 3, 1, 2, 4, 3])", "2"),
        ("(4, [1, 4, 4])", "1"),
        ("(11, [1, 1, 1, 1, 1, 1, 1, 1])", "0"),
    ])
    add("0210", "findOrder", [])  # Complex graph
    add("0211", "WordDictionary", [])  # Class design
    add("0212", "findWords", [])  # Complex
    add("0213", "rob", [
        ("([2, 3, 2],)", "3"),
        ("([1, 2, 3, 1],)", "4"),
        ("([1, 2, 3],)", "3"),
    ])
    add("0215", "findKthLargest", [
        ("([3, 2, 1, 5, 6, 4], 2)", "5"),
        ("([3, 2, 3, 1, 2, 4, 5, 5, 6], 4)", "4"),
    ])
    add("0217", "containsDuplicate", [
        ("([1, 2, 3, 1],)", "True"),
        ("([1, 2, 3, 4],)", "False"),
    ])
    add("0219", "containsNearbyDuplicate", [
        ("([1, 2, 3, 1], 3)", "True"),
        ("([1, 0, 1, 1], 1)", "True"),
        ("([1, 2, 3, 1, 2, 3], 2)", "False"),
    ])
    add("0221", "maximalSquare", [])  # 2D char matrix
    add("0225", "MyStack", [])  # Class design
    add("0226", "invertTree", [])  # TreeNode
    add("0230", "kthSmallest", [])  # TreeNode
    add("0231", "isPowerOfTwo", [
        ("(1,)", "True"),
        ("(16,)", "True"),
        ("(3,)", "False"),
    ])
    add("0232", "MyQueue", [])  # Class design
    add("0234", "isPalindrome", [])  # ListNode
    add("0235", "lowestCommonAncestor", [])  # TreeNode
    add("0236", "lowestCommonAncestor", [])  # TreeNode
    add("0239", "maxSlidingWindow", [])  # Complex deque
    add("0241", "diffWaysToCompute", [
        ('("2-1-1",)', "[0, 2]"),
        ('("2*3-4*5",)', "[-34, -14, -10, -10, 10]"),
    ])
    add("0242", "isAnagram", [
        ('("anagram", "nagaram")', "True"),
        ('("rat", "car")', "False"),
    ])
    add("0252", "canAttendMeetings", [
        ("([[0, 30], [5, 10], [15, 20]],)", "False"),
        ("([[7, 10], [2, 4]],)", "True"),
    ])
    add("0253", "minMeetingRooms", [])  # Heap
    add("0261", "validTree", [])  # Graph
    add("0263", "isUgly", [
        ("(6,)", "True"),
        ("(1,)", "True"),
        ("(14,)", "False"),
    ])
    add("0268", "missingNumber", [
        ("([3, 0, 1],)", "2"),
        ("([0, 1],)", "2"),
        ("([9, 6, 4, 2, 3, 5, 7, 0, 1],)", "8"),
    ])
    add("0269", "alienOrder", [])  # Complex graph
    add("0271", "encode_decode", [])  # Class design
    add("0274", "hIndex", [
        ("([3, 0, 6, 1, 5],)", "3"),
        ("([1, 3, 1],)", "1"),
    ])
    add("0278", "firstBadVersion", [])  # API call
    add("0280", "wiggleSort", [])  # In-place
    add("0283", "moveZeroes", [])  # In-place
    add("0286", "wallsAndGates", [])  # 2D in-place
    add("0287", "findDuplicate", [
        ("([1, 3, 4, 2, 2],)", "2"),
        ("([3, 1, 3, 4, 2],)", "3"),
    ])
    add("0290", "wordPattern", [
        ('("abba", "dog cat cat dog")', "True"),
        ('("abba", "dog cat cat fish")', "False"),
    ])
    add("0295", "MedianFinder", [])  # Class design + heap
    add("0297", "serialize_deserialize", [])  # TreeNode
    add("0303", "NumArray", [])  # Class design
    add("0304", "NumMatrix", [])  # Class design
    add("0309", "maxProfit", [
        ("([1, 2, 3, 0, 2],)", "3"),
        ("([1],)", "0"),
    ])
    add("0312", "maxCoins", [
        ("([3, 1, 5, 8],)", "167"),
        ("([1, 5],)", "10"),
    ])
    add("0322", "coinChange", [
        ("([1, 2, 5], 11)", "3"),
        ("([2], 3)", "-1"),
        ("([1], 0)", "0"),
    ])
    add("0323", "countComponents", [])  # Graph
    add("0329", "longestIncreasingPath", [])  # 2D matrix
    add("0332", "findItinerary", [])  # Complex graph
    add("0334", "increasingTriplet", [
        ("([1, 2, 3, 4, 5],)", "True"),
        ("([5, 4, 3, 2, 1],)", "False"),
        ("([2, 1, 5, 0, 4, 6],)", "True"),
    ])
    add("0338", "countBits", [
        ("(2,)", "[0, 1, 1]"),
        ("(5,)", "[0, 1, 1, 2, 1, 2]"),
    ])
    add("0344", "reverseString", [])  # In-place
    add("0347", "topKFrequent", [])  # Complex
    add("0349", "intersection", [])  # Set
    add("0350", "intersect", [])  # Counter
    add("0355", "Twitter", [])  # Class design
    add("0367", "isPerfectSquare", [
        ("(16,)", "True"),
        ("(14,)", "False"),
        ("(1,)", "True"),
    ])
    add("0371", "getSum", [])  # Bitwise
    add("0374", "guessNumber", [])  # API call
    add("0377", "combinationSum4", [
        ("([1, 2, 3], 4)", "7"),
    ])
    add("0380", "RandomizedSet", [])  # Class design
    add("0383", "canConstruct", [
        ('("a", "b")', "False"),
        ('("aa", "ab")', "False"),
        ('("aa", "aab")', "True"),
    ])
    add("0394", "decodeString", [
        ('("3[a]2[bc]",)', "aaabcbc"),
        ('("3[a2[c]]",)', "accaccacc"),
        ('("2[abc]3[cd]ef",)', "abcabccdcdcdef"),
    ])
    add("0402", "removeKdigits", [
        ('("1432219", 3)', "1219"),
        ('("10200", 1)', "200"),
        ('("10", 2)', "0"),
    ])
    add("0410", "splitArray", [
        ("([7, 2, 5, 10, 8], 2)", "18"),
        ("([1, 2, 3, 4, 5], 2)", "9"),
    ])
    add("0416", "canPartition", [
        ("([1, 5, 11, 5],)", "True"),
        ("([1, 2, 3, 5],)", "False"),
    ])
    add("0417", "pacificAtlantic", [])  # 2D grid
    add("0424", "characterReplacement", [
        ('("ABAB", 2)', "4"),
        ('("AABABBA", 1)', "4"),
    ])
    add("0435", "eraseOverlapIntervals", [
        ("([[1, 2], [2, 3], [3, 4], [1, 3]],)", "1"),
        ("([[1, 2], [1, 2], [1, 2]],)", "2"),
        ("([[1, 2], [2, 3]],)", "0"),
    ])
    add("0438", "findAnagrams", [
        ('("cbaebabacd", "abc")', "[0, 6]"),
        ('("abab", "ab")', "[0, 1, 2]"),
    ])
    add("0441", "arrangeCoins", [
        ("(5,)", "2"),
        ("(8,)", "3"),
    ])
    add("0442", "findDuplicates", [
        ("([4, 3, 2, 7, 8, 2, 3, 1],)", "[2, 3]"),
    ])
    add("0448", "findDisappearedNumbers", [
        ("([4, 3, 2, 7, 8, 2, 3, 1],)", "[5, 6]"),
    ])
    add("0450", "deleteNode", [])  # TreeNode
    add("0452", "findMinArrowShots", [
        ("([[10, 16], [2, 8], [1, 6], [7, 12]],)", "2"),
        ("([[1, 2], [3, 4], [5, 6], [7, 8]],)", "4"),
    ])
    add("0456", "find132pattern", [
        ("([1, 2, 3, 4],)", "False"),
        ("([3, 1, 4, 2],)", "True"),
        ("([-1, 3, 2, 0],)", "True"),
    ])
    add("0459", "repeatedSubstringPattern", [
        ('("abab",)', "True"),
        ('("aba",)', "False"),
        ('("abcabcabcabc",)', "True"),
    ])
    add("0463", "islandPerimeter", [])  # 2D grid
    add("0473", "makesquare", [])  # Complex backtracking
    add("0474", "findMaxForm", [])  # Complex
    add("0494", "findTargetSumWays", [
        ("([1, 1, 1, 1, 1], 3)", "5"),
    ])
    add("0496", "nextGreaterElement", [
        ("([4, 1, 2], [1, 3, 4, 2])", "[-1, 3, -1]"),
    ])
    add("0502", "findMaximizedCapital", [])  # Heap
    add("0513", "findBottomLeftValue", [])  # TreeNode
    add("0516", "longestPalindromeSubseq", [
        ('("bbbab",)', "4"),
        ('("cbbd",)', "2"),
    ])
    add("0518", "change", [
        ("(5, [1, 2, 5])", "4"),
        ("(3, [2])", "0"),
        ("(10, [10])", "1"),
    ])
    add("0523", "checkSubarraySum", [
        ("([23, 2, 4, 6, 7], 6)", "True"),
        ("([23, 2, 6, 4, 7], 6)", "True"),
        ("([23, 2, 6, 4, 7], 13)", "False"),
    ])
    add("0525", "findMaxLength", [
        ("([0, 1],)", "2"),
        ("([0, 1, 0],)", "2"),
    ])
    add("0535", "encode_decode_tinyurl", [])  # Class design
    add("0540", "singleNonDuplicate", [
        ("([1, 1, 2, 3, 3, 4, 4, 8, 8],)", "2"),
        ("([3, 3, 7, 7, 10, 11, 11],)", "10"),
    ])
    add("0543", "diameterOfBinaryTree", [])  # TreeNode
    add("0554", "leastBricks", [])  # Complex
    add("0560", "subarraySum", [
        ("([1, 1, 1], 2)", "2"),
        ("([1, 2, 3], 3)", "2"),
    ])
    add("0567", "checkInclusion", [
        ('("ab", "eidbaooo")', "True"),
        ('("ab", "eidboaoo")', "False"),
    ])
    add("0572", "isSubtree", [])  # TreeNode
    add("0606", "tree2str", [])  # TreeNode
    add("0617", "mergeTrees", [])  # TreeNode
    add("0621", "leastInterval", [])  # Complex
    add("0622", "MyCircularQueue", [])  # Class design
    add("0647", "countSubstrings", [
        ('("abc",)', "3"),
        ('("aaa",)', "6"),
    ])
    add("0658", "findClosestElements", [
        ("([1, 2, 3, 4, 5], 4, 3)", "[1, 2, 3, 4]"),
    ])
    add("0662", "widthOfBinaryTree", [])  # TreeNode
    add("0665", "checkPossibility", [
        ("([4, 2, 3],)", "True"),
        ("([4, 2, 1],)", "False"),
        ("([3, 4, 2, 3],)", "False"),
    ])
    add("0669", "trimBST", [])  # TreeNode
    add("0673", "findNumberOfLIS", [
        ("([1, 3, 5, 4, 7],)", "2"),
        ("([2, 2, 2, 2, 2],)", "5"),
    ])
    add("0678", "checkValidString", [
        ('("()",)', "True"),
        ('("(*)",)', "True"),
        ('("(*))",)', "True"),
    ])
    add("0680", "validPalindrome", [
        ('("aba",)', "True"),
        ('("abca",)', "True"),
        ('("abc",)', "False"),
    ])
    add("0682", "calPoints", [
        ('(["5", "2", "C", "D", "+"],)', "30"),
    ])
    add("0684", "findRedundantConnection", [])  # Union-Find
    add("0695", "maxAreaOfIsland", [])  # 2D grid
    add("0698", "canPartitionKSubsets", [])  # Complex backtracking
    add("0701", "insertIntoBST", [])  # TreeNode
    add("0703", "KthLargest", [])  # Class design + heap
    add("0705", "MyHashSet", [])  # Class design
    add("0706", "MyHashMap", [])  # Class design
    add("0707", "MyLinkedList", [])  # Class design
    add("0721", "accountsMerge", [])  # Complex
    add("0724", "pivotIndex", [
        ("([1, 7, 3, 6, 5, 6],)", "3"),
        ("([1, 2, 3],)", "-1"),
        ("([2, 1, -1],)", "0"),
    ])
    add("0729", "MyCalendar", [])  # Class design
    add("0735", "asteroidCollision", [
        ("([5, 10, -5],)", "[5, 10]"),
        ("([8, -8],)", "[]"),
        ("([10, 2, -5],)", "[10]"),
    ])
    add("0739", "dailyTemperatures", [
        ("([73, 74, 75, 71, 69, 72, 76, 73],)", "[1, 1, 4, 2, 1, 1, 0, 0]"),
    ])
    add("0740", "deleteAndEarn", [
        ("([3, 4, 2],)", "6"),
        ("([2, 2, 3, 3, 3, 4],)", "9"),
    ])
    add("0743", "networkDelayTime", [])  # Graph + heap
    add("0745", "WordFilter", [])  # Class design
    add("0746", "minCostClimbingStairs", [
        ("([10, 15, 20],)", "15"),
        ("([1, 100, 1, 1, 1, 100, 1, 1, 100, 1],)", "6"),
    ])
    add("0752", "openLock", [])  # BFS
    add("0763", "partitionLabels", [
        ('("ababcbacadefegdehijhklij",)', "[9, 7, 8]"),
    ])
    add("0767", "reorganizeString", [])  # Heap
    add("0778", "swimInWater", [])  # Heap + grid
    add("0783", "minDiffInBST", [])  # TreeNode
    add("0785", "isBipartite", [])  # Graph
    add("0787", "findCheapestPrice", [])  # Graph
    add("0791", "customSortString", [
        ('("cba", "abcd")', "cbad"),
    ])
    add("0802", "eventualSafeNodes", [])  # Graph
    add("0838", "pushDominoes", [])  # Complex
    add("0846", "isNStraightHand", [])  # Counter + heap
    add("0853", "carFleet", [
        ("(12, [10, 8, 0, 5, 3], [2, 4, 1, 1, 3])", "3"),
    ])
    add("0862", "shortestSubarray", [])  # Deque
    add("0875", "minEatingSpeed", [
        ("([3, 6, 7, 11], 8)", "4"),
        ("([30, 11, 23, 4, 20], 5)", "30"),
        ("([30, 11, 23, 4, 20], 6)", "23"),
    ])
    add("0876", "middleNode", [])  # ListNode
    add("0881", "numRescueBoats", [
        ("([1, 2], 3)", "1"),
        ("([3, 2, 2, 1], 3)", "3"),
        ("([3, 5, 3, 4], 5)", "4"),
    ])
    add("0894", "allPossibleFBT", [])  # TreeNode
    add("0895", "FreqStack", [])  # Class design
    add("0896", "isMonotonic", [
        ("([1, 2, 2, 3],)", "True"),
        ("([6, 5, 4, 4],)", "True"),
        ("([1, 3, 2],)", "False"),
    ])
    add("0901", "StockSpanner", [])  # Class design
    add("0904", "totalFruit", [])  # Complex
    add("0909", "snakesAndLadders", [])  # BFS
    add("0912", "sortArray", [
        ("([5, 2, 3, 1],)", "[1, 2, 3, 5]"),
        ("([5, 1, 1, 2, 0, 0],)", "[0, 0, 1, 1, 2, 5]"),
    ])
    add("0918", "maxSubarraySumCircular", [
        ("([1, -2, 3, -2],)", "3"),
        ("([5, -3, 5],)", "10"),
        ("([-3, -2, -3],)", "-2"),
    ])
    add("0929", "numUniqueEmails", [])  # Set + string
    add("0930", "numSubarraysWithSum", [
        ("([1, 0, 1, 0, 1], 2)", "4"),
    ])
    add("0931", "minFallingPathSum", [])  # 2D DP
    add("0946", "validateStackSequences", [
        ("([1, 2, 3, 4, 5], [4, 5, 3, 2, 1])", "True"),
        ("([1, 2, 3, 4, 5], [4, 3, 5, 1, 2])", "False"),
    ])
    add("0948", "bagOfTokensScore", [
        ("([100], 50)", "0"),
        ("([200, 100], 150)", "1"),
        ("([100, 200, 300, 400], 200)", "2"),
    ])
    add("0953", "isAlienSorted", [])  # Complex
    add("0973", "kClosest", [])  # Heap
    add("0977", "sortedSquares", [
        ("([-4, -1, 0, 3, 10],)", "[0, 1, 9, 16, 100]"),
        ("([-7, -3, 2, 3, 11],)", "[4, 9, 9, 49, 121]"),
    ])
    add("0978", "maxTurbulenceSize", [
        ("([9, 4, 2, 10, 7, 8, 8, 1, 9],)", "5"),
        ("([4, 8, 12, 16],)", "2"),
        ("([100],)", "1"),
    ])
    add("0981", "TimeMap", [])  # Class design
    add("0994", "orangesRotting", [])  # BFS 2D
    add("0997", "findJudge", [
        ("(2, [[1, 2]],)", "2"),
        ("(3, [[1, 3], [2, 3]],)", "3"),
        ("(3, [[1, 3], [2, 3], [3, 1]],)", "-1"),
    ])
    add("1011", "shipWithinDays", [
        ("([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 5)", "15"),
        ("([3, 2, 2, 4, 1, 4], 3)", "6"),
    ])
    add("1020", "numEnclaves", [])  # 2D grid
    add("1029", "twoCitySchedCost", [
        ("([[10, 20], [30, 200], [400, 50], [30, 20]],)", "110"),
    ])
    add("1046", "lastStoneWeight", [])  # Heap
    add("1049", "lastStoneWeightII", [
        ("([2, 7, 4, 1, 8, 1],)", "1"),
    ])
    add("1074", "numSubmatrixSumTarget", [])  # Complex
    add("1091", "shortestPathBinaryMatrix", [])  # BFS
    add("1137", "tribonacci", [
        ("(4,)", "4"),
        ("(25,)", "1389537"),
    ])
    add("1143", "longestCommonSubsequence", [
        ('("abcde", "ace")', "3"),
        ('("abc", "abc")', "3"),
        ('("abc", "def")', "0"),
    ])
    add("1189", "maxNumberOfBalloons", [])  # Counter
    add("1203", "sortItems", [])  # Complex graph
    add("1209", "removeDuplicates", [
        ('("abcd", 2)', "abcd"),
        ('("deeedbbcccbdaa", 3)', "aa"),
        ('("pbbcggttciiippooaais", 2)', "ps"),
    ])
    add("1220", "countVowelPermutation", [
        ("(1,)", "5"),
        ("(2,)", "10"),
        ("(5,)", "68"),
    ])
    add("1239", "maxLength", [])  # Complex
    add("1254", "closedIsland", [])  # 2D grid
    add("1260", "shiftGrid", [])  # 2D grid
    add("1288", "removeCoveredIntervals", [
        ("([[1, 4], [3, 6], [2, 8]],)", "2"),
    ])
    add("1299", "replaceElements", [
        ("([17, 18, 5, 4, 6, 1],)", "[18, 6, 6, 6, 1, -1]"),
    ])
    add("1343", "numOfSubarrays", [
        ("([2, 1, 5, 6, 0, 9, 8], 3, 4)", "3"),
    ])
    add("1345", "minJumps", [])  # BFS
    add("1383", "maxPerformance", [])  # Heap
    add("1396", "UndergroundSystem", [])  # Class design
    add("1397", "findGoodStrings", [])  # Complex
    add("1423", "maxScore", [
        ("([1, 2, 3, 4, 5, 6, 1], 3)", "12"),
        ("([2, 2, 2], 2)", "4"),
        ("([9, 7, 7, 9, 7, 7, 9], 7)", "55"),
    ])
    add("1448", "goodNodes", [])  # TreeNode
    add("1456", "maxVowels", [
        ('("abciiidef", 3)', "3"),
        ('("aeiou", 2)', "2"),
        ('("leetcode", 3)', "2"),
    ])
    add("1461", "hasAllCodes", [])  # Set
    add("1462", "courseScheduleIV", [])  # Graph
    add("1464", "maxProduct", [
        ("([3, 4, 5, 2],)", "12"),
        ("([1, 5, 4, 5],)", "16"),
        ("([3, 7],)", "12"),
    ])
    add("1466", "minReorder", [])  # Graph
    add("1472", "BrowserHistory", [])  # Class design
    add("1475", "finalPrices", [
        ("([8, 4, 6, 2, 3],)", "[4, 2, 4, 2, 3]"),
        ("([1, 2, 3, 4, 5],)", "[1, 2, 3, 4, 5]"),
    ])
    add("1481", "findLeastNumOfUniqueInts", [])  # Counter + heap
    add("1489", "findCriticalAndPseudoCriticalEdges", [])  # Complex
    add("1498", "numSubseq", [])  # Complex
    add("1514", "maxProbability", [])  # Graph + heap
    add("1523", "countOdds", [
        ("(3, 7)", "3"),
        ("(8, 10)", "1"),
    ])
    add("1572", "diagonalSum", [
        ("([[1, 2, 3], [4, 5, 6], [7, 8, 9]],)", "25"),
        ("([[1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1], [1, 1, 1, 1]],)", "8"),
    ])
    add("1582", "numSpecial", [])  # 2D matrix
    add("1584", "minCostConnectPoints", [])  # Heap
    add("1603", "ParkingSystem", [])  # Class design
    add("1609", "isEvenOddTree", [])  # TreeNode
    add("1631", "minimumEffortPath", [])  # Heap + grid
    add("1642", "furthestBuilding", [])  # Heap
    add("1658", "minOperations", [
        ("([1, 1, 4, 2, 3], 5)", "2"),
        ("([5, 6, 7, 8, 9], 4)", "-1"),
        ("([3, 2, 20, 1, 1, 3], 10)", "5"),
    ])
    add("1669", "mergeInBetween", [])  # ListNode
    add("1700", "countStudents", [
        ("([1, 1, 0, 0], [0, 1, 0, 1])", "0"),
        ("([1, 1, 1, 0, 0, 1], [1, 0, 0, 0, 1, 1])", "3"),
    ])
    add("1721", "swapNodes", [])  # ListNode
    add("1750", "minimumLength", [
        ('("ca",)', "2"),
        ('("cabaabac",)', "0"),
        ('("aabccabba",)', "3"),
    ])
    add("1800", "maxAscendingSum", [
        ("([10, 20, 30, 5, 10, 50],)", "65"),
        ("([10, 20, 30, 40, 50],)", "150"),
        ("([12, 17, 15, 13, 10, 11, 12],)", "33"),
    ])
    add("1822", "arraySign", [
        ("([-1, -2, -3, -4, 3, 2, 1],)", "1"),
        ("([1, 5, 0, 2, -3],)", "0"),
        ("([-1, 1, -1, 1, -1],)", "-1"),
    ])
    add("1834", "getOrder", [])  # Heap
    add("1838", "maxFrequency", [])  # Complex
    add("1845", "SeatManager", [])  # Heap
    add("1849", "splitString", [
        ('("1234",)', "False"),
        ('("050043",)', "True"),
        ('("9080701",)', "False"),
    ])
    add("1851", "minInterval", [])  # Heap
    add("1888", "minFlips", [])  # Complex
    add("1899", "mergeTriplets", [
        ("([[2, 5, 3], [1, 8, 4], [1, 7, 5]], [2, 7, 5])", "True"),
        ("([[3, 4, 5], [4, 5, 6]], [3, 2, 5])", "False"),
    ])
    add("1905", "countSubIslands", [])  # 2D grid
    add("1930", "countPalindromicSubsequence", [])  # Complex
    add("1958", "checkMove", [])  # 2D grid
    add("1963", "minSwaps", [
        ('("][][",)', "1"),
        ('("[][][]",)', "0"),
        ('("]]][[[",)', "2"),
    ])
    add("1968", "rearrangeArray", [
        ("([3, 1, -2, -5, 2, -4],)", "[3, -2, 1, -5, 2, -4]"),
    ])
    add("1980", "findDifferentBinaryString", [
        ('(["01", "10"],)', "00"),
    ])
    add("1984", "minimumDifference", [
        ("([90], 1)", "0"),
    ])
    add("1985", "kthLargestNumber", [])  # Heap
    add("2001", "interchangeableRectangles", [])  # Complex
    add("2002", "maxProduct", [])  # Complex
    add("2013", "DetectSquares", [])  # Class design
    add("2017", "gridGame", [])  # 2D prefix sum
    add("2092", "findAllPeople", [])  # Complex graph
    add("2101", "maximumDetonation", [])  # Graph
    add("2130", "pairSum", [])  # ListNode
    add("2215", "findDifference", [
        ("([1, 2, 3], [2, 4, 6])", "[[1, 3], [4, 6]]"),
    ])
    add("2235", "sum", [
        ("(12, 5)", "17"),
        ("(-10, 4)", "-6"),
    ])
    add("2300", "successfulPairs", [
        ("([5, 1, 3], [1, 2, 3, 4, 5], 7)", "[4, 0, 3]"),
    ])
    add("2306", "distinctNames", [])  # Complex
    add("2348", "zeroFilledSubarray", [
        ("([1, 3, 0, 0, 2, 0, 0, 4],)", "6"),
        ("([0, 0, 0, 2, 0, 0],)", "9"),
        ("([2, 10, 2019],)", "0"),
    ])
    add("2390", "removeStars", [
        ('("leet**cod*e",)', "lecoe"),
        ('("erase*****",)', ""),
    ])
    add("2402", "mostBooked", [])  # Heap
    add("2405", "minPartitions", [
        ('("abacbc",)', "3"),
        ('("ssssss",)', "1"),
    ])
    add("2482", "onesMinusZeros", [])  # 2D matrix
    add("2483", "bestClosingTime", [
        ('("YYNY",)', "2"),
        ('("NNNNN",)', "0"),
        ('("YYYY",)', "4"),
    ])
    add("2554", "maxCount", [
        ("(6, [1, 6, 5], 2)", "3"),
    ])
    add("2616", "minimizeMax", [
        ("([10, 1, 2, 7, 1, 3], 2)", "1"),
        ("([4, 2, 1, 2], 1)", "0"),
    ])
    add("2709", "canTraverseAllPairs", [])  # Complex
    add("2864", "maximumOddBinaryNumber", [
        ('("010",)', "001"),
        ('("0101",)', "1001"),
    ])
    add("2971", "largestPerimeter", [
        ("([5, 5, 5],)", "15"),
        ("([1, 12, 1, 2, 5, 50, 3],)", "12"),
        ("([5, 5, 50],)", "-1"),
    ])
    
    return cases


# ─── Conversion logic ─────────────────────────────────────────────────

def filename_to_num(filename):
    """Extract problem number from filename like '0001-two-sum.py'"""
    return filename[:4]

def filename_to_slug(filename):
    """Extract slug from filename like '0001-two-sum.py' -> '0001_two_sum'"""
    name = filename.replace(".py", "").replace("-", "_")
    return name

def read_python_source(filepath):
    """Read and return the Python source code."""
    with open(filepath, "r") as f:
        return f.read()

def extract_methods(source):
    """Extract method definitions from a Solution class."""
    methods = []
    lines = source.split("\n")
    in_method = False
    method_lines = []
    indent = 0
    
    for line in lines:
        if re.match(r'\s+def\s+\w+\s*\(', line) and 'self' in line:
            if in_method and method_lines:
                methods.append("\n".join(method_lines))
            in_method = True
            method_lines = [line]
            indent = len(line) - len(line.lstrip())
        elif in_method:
            if line.strip() == "" or (len(line) - len(line.lstrip()) > indent) or line.strip().startswith("#"):
                method_lines.append(line)
            elif line.strip().startswith("def ") or line.strip().startswith("class "):
                methods.append("\n".join(method_lines))
                in_method = False
                method_lines = []
            else:
                method_lines.append(line)
    
    if in_method and method_lines:
        methods.append("\n".join(method_lines))
    
    return methods

def convert_method_to_function(method_text):
    """Convert a class method to a standalone function."""
    lines = method_text.split("\n")
    result = []
    
    for i, line in enumerate(lines):
        # Remove self parameter
        line = re.sub(r'\(self,\s*', '(', line)
        line = re.sub(r'\(self\)', '()', line)
        # Remove leading class indentation (typically 4 spaces)
        if line.startswith("    "):
            line = line[4:]
        elif line.startswith("\t"):
            line = line[1:]
        # Remove self. references
        line = line.replace("self.", "")
        result.append(line)
    
    return "\n".join(result)

def python_to_sifr_function(func_text):
    """Convert Python function text to Sifr-compatible code."""
    # Replace List[int] etc with list[int]
    text = func_text
    text = re.sub(r'List\[(\w+)\]', r'list[\1]', text)
    text = re.sub(r'List\[List\[(\w+)\]\]', r'list[list[\1]]', text)
    # Remove docstrings
    text = re.sub(r'"""[^"]*"""', '', text)
    text = re.sub(r"'''[^']*'''", '', text)
    return text


def generate_sifr_file(problem_num, problem_name, python_source, func_name, tests):
    """Generate a Sifr file from Python source and test cases."""
    # Extract and convert the main method
    methods = extract_methods(python_source)
    if not methods:
        # Maybe it's already a standalone function
        return None
    
    # Take the first (main) method
    main_method = methods[0]
    func_text = convert_method_to_function(main_method)
    sifr_func = python_to_sifr_function(func_text)
    
    # Build test calls
    test_lines = []
    expect_lines = []
    for args, expected in tests:
        # Clean up args - remove outer parens if single arg tuple
        clean_args = args.strip()
        if clean_args.startswith("(") and clean_args.endswith(")"):
            clean_args = clean_args[1:-1]
            # Remove trailing comma for single-arg tuples
            if clean_args.endswith(","):
                clean_args = clean_args[:-1]
        test_lines.append(f"    print({func_name}({clean_args}))")
        # Convert Python bools to Sifr bools
        sifr_expected = expected.replace("True", "true").replace("False", "false")
        expect_lines.append(f"# expect-stdout: {sifr_expected}")
    
    content = f"# LeetCode {int(problem_num)}: {problem_name}\n\n"
    content += sifr_func.rstrip() + "\n\n"
    content += "def main():\n"
    content += "\n".join(test_lines) + "\n\n"
    content += "\n".join(expect_lines) + "\n"
    
    return content


def generate_python_file(problem_num, problem_name, python_source, func_name, tests):
    """Generate a Python test file from source and test cases."""
    methods = extract_methods(python_source)
    if not methods:
        return None
    
    main_method = methods[0]
    func_text = convert_method_to_function(main_method)
    
    test_lines = []
    for args, expected in tests:
        clean_args = args.strip()
        if clean_args.startswith("(") and clean_args.endswith(")"):
            clean_args = clean_args[1:-1]
            if clean_args.endswith(","):
                clean_args = clean_args[:-1]
        test_lines.append(f"    print({func_name}({clean_args}))")
    
    content = f"# LeetCode {int(problem_num)}: {problem_name}\n# Python version with test cases\n\n"
    content += func_text.rstrip() + "\n\n"
    content += "def main():\n"
    content += "\n".join(test_lines) + "\n\n"
    content += 'if __name__ == "__main__":\n    main()\n'
    
    return content


def get_problem_name(filename):
    """Extract human-readable problem name from filename."""
    name = filename.replace(".py", "")
    # Remove number prefix
    name = re.sub(r'^\d+-', '', name)
    # Convert dashes to spaces and title case
    return name.replace("-", " ").title()


def main():
    test_cases = make_test_cases()
    
    # Get all Python files
    py_files = sorted(os.listdir(PYTHON_DIR))
    
    stats = {"total": 0, "converted": 0, "skipped_done": 0, "skipped_no_tests": 0, "skipped_complex": 0}
    
    for filename in py_files:
        if not filename.endswith(".py"):
            continue
        
        num = filename_to_num(filename)
        slug = filename_to_slug(filename)
        
        stats["total"] += 1
        
        # Skip already done
        if num in ALREADY_DONE:
            stats["skipped_done"] += 1
            continue
        
        # Check if we have test cases
        if num not in test_cases:
            stats["skipped_no_tests"] += 1
            continue
        
        func_name, tests = test_cases[num]
        
        if not tests:
            stats["skipped_complex"] += 1
            continue
        
        # Read Python source
        filepath = os.path.join(PYTHON_DIR, filename)
        python_source = read_python_source(filepath)
        problem_name = get_problem_name(filename)
        
        # Generate Sifr file
        sifr_content = generate_sifr_file(num, problem_name, python_source, func_name, tests)
        if sifr_content is None:
            stats["skipped_complex"] += 1
            continue
        
        # Generate Python file
        py_content = generate_python_file(num, problem_name, python_source, func_name, tests)
        if py_content is None:
            stats["skipped_complex"] += 1
            continue
        
        # Write files
        sifr_path = os.path.join(OUTPUT_DIR, f"{slug}.sifr")
        py_path = os.path.join(OUTPUT_DIR, f"{slug}.py")
        
        with open(sifr_path, "w") as f:
            f.write(sifr_content)
        with open(py_path, "w") as f:
            f.write(py_content)
        
        stats["converted"] += 1
        print(f"  Converted: {slug}")
    
    print(f"\n=== Conversion Stats ===")
    print(f"Total files: {stats['total']}")
    print(f"Already done: {stats['skipped_done']}")
    print(f"Converted: {stats['converted']}")
    print(f"Skipped (no tests / complex): {stats['skipped_no_tests'] + stats['skipped_complex']}")


if __name__ == "__main__":
    main()
