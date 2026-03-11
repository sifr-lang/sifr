#!/usr/bin/env python3
"""
LeetCode → Sifr Audit Runner

For each of the 376 remaining LeetCode problems:
1. Read the Python source
2. Convert to Sifr (best-effort automatic conversion)
3. Add test cases
4. Write .py and .sifr files
5. Compile the .sifr file with `sifr run`
6. Record: PASS (compiles + correct output), COMPILE_ERROR (type/syntax error), RUNTIME_ERROR, SKIP

The conversion rules:
- class Solution → standalone function
- self removed
- List[int] → list[int]
- Type annotations preserved
- No set() → use dict workaround
- No enumerate tuple unpacking → use range(len())
- Safe indexing: list[i] returns T | None
"""
import os
import re
import subprocess
import sys
import json

SRC = "/Users/yaseralnajjar/work/sifr/leetcode/python"
DST = "/Users/yaseralnajjar/work/sifr/codebase/audit/leetcode"
SIFR_BIN = "/Users/yaseralnajjar/work/sifr/codebase/target/release/sifr"
WORKSPACE = "/Users/yaseralnajjar/work/sifr/codebase"

DONE = {
    "0009","0014","0045","0053","0055","0058","0070","0121",
    "0134","0152","0169","0198","0238","0300","0392","0509",
    "0605","0704","1768","1929",
}

# ─── Test cases for each problem ──────────────────────────────────────
# Format: { "NNNN": (func_name, [(py_call_args, expected_output), ...]) }
# py_call_args: string of arguments as in func(args)
# expected_output: string of expected print output

TEST_CASES = {
    "0001": ("twoSum", [("[2,7,11,15], 9", "[0, 1]"), ("[3,2,4], 6", "[1, 2]"), ("[3,3], 6", "[0, 1]")]),
    "0003": ("lengthOfLongestSubstring", [('\"abcabcbb\"', "3"), ('\"bbbbb\"', "1"), ('\"pwwkew\"', "3")]),
    "0004": ("findMedianSortedArrays", [("[1,3], [2]", "2.0"), ("[1,2], [3,4]", "2.5")]),
    "0005": ("longestPalindrome", [('\"babad\"', "bab"), ('\"cbbd\"', "bb")]),
    "0006": ("convert", [('\"PAYPALISHIRING\", 3', "PAHNAPLSIIGYIR"), ('\"A\", 1', "A")]),
    "0007": ("reverse", [("123", "321"), ("-123", "-321"), ("120", "21"), ("0", "0")]),
    "0011": ("maxArea", [("[1,8,6,2,5,4,8,3,7]", "49"), ("[1,1]", "1")]),
    "0012": ("intToRoman", [("3749", "MMMDCCXLIX"), ("58", "LVIII"), ("1994", "MCMXCIV")]),
    "0013": ("romanToInt", [('\"III\"', "3"), ('\"LVIII\"', "58"), ('\"MCMXCIV\"', "1994")]),
    "0015": ("threeSum", [("[-1,0,1,2,-1,-4]", "[[-1, -1, 2], [-1, 0, 1]]")]),
    "0016": ("threeSumClosest", [("[-1,2,1,-4], 1", "2")]),
    "0017": ("letterCombinations", [('\"23\"', '["ad", "ae", "af", "bd", "be", "bf", "cd", "ce", "cf"]')]),
    "0018": ("fourSum", [("[1,0,-1,0,-2,2], 0", "[[-2, -1, 1, 2], [-2, 0, 0, 2], [-1, 0, 0, 1]]")]),
    "0020": ("isValid", [('\"()\"', "True"), ('\"()[]{}\"', "True"), ('\"(]\"', "False")]),
    "0022": ("generateParenthesis", [("3", '["((()))", "(()())", "(())()", "()(())", "()()()"]')]),
    "0026": ("removeDuplicates", [("[1,1,2]", "2"), ("[0,0,1,1,1,2,2,3,3,4]", "5")]),
    "0027": ("removeElement", [("[3,2,2,3], 3", "2"), ("[0,1,2,2,3,0,4,2], 2", "5")]),
    "0028": ("strStr", [('\"sadbutsad\", \"sad\"', "0"), ('\"leetcode\", \"leeto\"', "-1")]),
    "0033": ("search", [("[4,5,6,7,0,1,2], 0", "4"), ("[4,5,6,7,0,1,2], 3", "-1"), ("[1], 0", "-1")]),
    "0034": ("searchRange", [("[5,7,7,8,8,10], 8", "[3, 4]"), ("[5,7,7,8,8,10], 6", "[-1, -1]")]),
    "0035": ("searchInsert", [("[1,3,5,6], 5", "2"), ("[1,3,5,6], 2", "1"), ("[1,3,5,6], 7", "4")]),
    "0039": ("combinationSum", [("[2,3,6,7], 7", "[[2, 2, 3], [7]]")]),
    "0040": ("combinationSum2", [("[10,1,2,7,6,1,5], 8", "[[1, 1, 6], [1, 2, 5], [1, 7], [2, 6]]")]),
    "0041": ("firstMissingPositive", [("[1,2,0]", "3"), ("[3,4,-1,1]", "2"), ("[7,8,9,11,12]", "1")]),
    "0042": ("trap", [("[0,1,0,2,1,0,1,3,2,1,2,1]", "6"), ("[4,2,0,3,2,5]", "9")]),
    "0043": ("multiply", [('\"2\", \"3\"', "6"), ('\"123\", \"456\"', "56088")]),
    "0044": ("isMatch", [('\"aa\", \"a\"', "False"), ('\"aa\", \"*\"', "True"), ('\"cb\", \"?a\"', "False")]),
    "0046": ("permute", [("[1,2,3]", "[[1, 2, 3], [1, 3, 2], [2, 1, 3], [2, 3, 1], [3, 1, 2], [3, 2, 1]]")]),
    "0047": ("permuteUnique", [("[1,1,2]", "[[1, 1, 2], [1, 2, 1], [2, 1, 1]]")]),
    "0050": ("myPow", [("2.0, 10", "1024.0"), ("2.0, -2", "0.25")]),
    "0052": ("totalNQueens", [("4", "2"), ("1", "1")]),
    "0054": ("spiralOrder", [("[[1,2,3],[4,5,6],[7,8,9]]", "[1, 2, 3, 6, 9, 8, 7, 4, 5]")]),
    "0056": ("merge", [("[[1,3],[2,6],[8,10],[15,18]]", "[[1, 6], [8, 10], [15, 18]]")]),
    "0062": ("uniquePaths", [("3, 7", "28"), ("3, 2", "3")]),
    "0063": ("uniquePathsWithObstacles", [("[[0,0,0],[0,1,0],[0,0,0]]", "2")]),
    "0064": ("minPathSum", [("[[1,3,1],[1,5,1],[4,2,1]]", "7")]),
    "0066": ("plusOne", [("[1,2,3]", "[1, 2, 4]"), ("[9]", "[1, 0]")]),
    "0067": ("addBinary", [('\"11\", \"1\"', "100"), ('\"1010\", \"1011\"', "10101")]),
    "0069": ("mySqrt", [("4", "2"), ("8", "2"), ("0", "0")]),
    "0072": ("minDistance", [('\"horse\", \"ros\"', "3"), ('\"intention\", \"execution\"', "5")]),
    "0074": ("searchMatrix", [("[[1,3,5,7],[10,11,16,20],[23,30,34,60]], 3", "True"), ("[[1,3,5,7],[10,11,16,20],[23,30,34,60]], 13", "False")]),
    "0077": ("combine", [("4, 2", "[[1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]")]),
    "0078": ("subsets", [("[1,2,3]", "[[], [1], [1, 2], [1, 2, 3], [1, 3], [2], [2, 3], [3]]")]),
    "0080": ("removeDuplicates", [("[1,1,1,2,2,3]", "5")]),
    "0081": ("search", [("[2,5,6,0,0,1,2], 0", "True"), ("[2,5,6,0,0,1,2], 3", "False")]),
    "0084": ("largestRectangleArea", [("[2,1,5,6,2,3]", "10"), ("[2,4]", "4")]),
    "0090": ("subsetsWithDup", [("[1,2,2]", "[[], [1], [1, 2], [1, 2, 2], [2], [2, 2]]")]),
    "0091": ("numDecodings", [('\"12\"', "2"), ('\"226\"', "3"), ('\"06\"', "0")]),
    "0097": ("isInterleave", [('\"aabcc\", \"dbbca\", \"aadbbcbcac\"', "True"), ('\"aabcc\", \"dbbca\", \"aadbbbaccc\"', "False")]),
    "0115": ("numDistinct", [('\"rabbbit\", \"rabbit\"', "3"), ('\"babgbag\", \"bag\"', "5")]),
    "0118": ("generate", [("5", "[[1], [1, 1], [1, 2, 1], [1, 3, 3, 1], [1, 4, 6, 4, 1]]")]),
    "0119": ("getRow", [("3", "[1, 3, 3, 1]"), ("0", "[1]")]),
    "0120": ("minimumTotal", [("[[2],[3,4],[6,5,7],[4,1,8,3]]", "11")]),
    "0122": ("maxProfit", [("[7,1,5,3,6,4]", "7"), ("[1,2,3,4,5]", "4"), ("[7,6,4,3,1]", "0")]),
    "0125": ("isPalindrome", [('\"A man, a plan, a canal: Panama\"', "True"), ('\"race a car\"', "False")]),
    "0128": ("longestConsecutive", [("[100,4,200,1,3,2]", "4")]),
    "0131": ("partition", [('\"aab\"', '[["a", "a", "b"], ["aa", "b"]]')]),
    "0135": ("candy", [("[1,0,2]", "5"), ("[1,2,2]", "4")]),
    "0136": ("singleNumber", [("[2,2,1]", "1"), ("[4,1,2,1,2]", "4")]),
    "0139": ("wordBreak", [('\"leetcode\", [\"leet\",\"code\"]', "True"), ('\"applepenapple\", [\"apple\",\"pen\"]', "True")]),
    "0150": ("evalRPN", [('[\"2\",\"1\",\"+\",\"3\",\"*\"]', "9")]),
    "0151": ("reverseWords", [('\"the sky is blue\"', "blue is sky the")]),
    "0153": ("findMin", [("[3,4,5,1,2]", "1"), ("[4,5,6,7,0,1,2]", "0")]),
    "0162": ("findPeakElement", [("[1,2,3,1]", "2")]),
    "0167": ("twoSum", [("[2,7,11,15], 9", "[1, 2]"), ("[2,3,4], 6", "[1, 3]")]),
    "0168": ("convertToTitle", [("1", "A"), ("28", "AB"), ("701", "ZY")]),
    "0179": ("largestNumber", [("[10,2]", "210"), ("[3,30,34,5,9]", "9534330")]),
    "0202": ("isHappy", [("19", "True"), ("2", "False")]),
    "0205": ("isIsomorphic", [('\"egg\", \"add\"', "True"), ('\"foo\", \"bar\"', "False")]),
    "0209": ("minSubArrayLen", [("7, [2,3,1,2,4,3]", "2"), ("4, [1,4,4]", "1"), ("11, [1,1,1,1,1,1,1,1]", "0")]),
    "0213": ("rob", [("[2,3,2]", "3"), ("[1,2,3,1]", "4")]),
    "0217": ("containsDuplicate", [("[1,2,3,1]", "True"), ("[1,2,3,4]", "False")]),
    "0219": ("containsNearbyDuplicate", [("[1,2,3,1], 3", "True"), ("[1,2,3,1,2,3], 2", "False")]),
    "0231": ("isPowerOfTwo", [("1", "True"), ("16", "True"), ("3", "False")]),
    "0241": ("diffWaysToCompute", [('\"2-1-1\"', "[0, 2]")]),
    "0242": ("isAnagram", [('\"anagram\", \"nagaram\"', "True"), ('\"rat\", \"car\"', "False")]),
    "0252": ("canAttendMeetings", [("[[0,30],[5,10],[15,20]]", "False"), ("[[7,10],[2,4]]", "True")]),
    "0263": ("isUgly", [("6", "True"), ("1", "True"), ("14", "False")]),
    "0268": ("missingNumber", [("[3,0,1]", "2"), ("[0,1]", "2")]),
    "0274": ("hIndex", [("[3,0,6,1,5]", "3")]),
    "0287": ("findDuplicate", [("[1,3,4,2,2]", "2"), ("[3,1,3,4,2]", "3")]),
    "0290": ("wordPattern", [('\"abba\", \"dog cat cat dog\"', "True"), ('\"abba\", \"dog cat cat fish\"', "False")]),
    "0309": ("maxProfit", [("[1,2,3,0,2]", "3"), ("[1]", "0")]),
    "0312": ("maxCoins", [("[3,1,5,8]", "167")]),
    "0322": ("coinChange", [("[1,2,5], 11", "3"), ("[2], 3", "-1"), ("[1], 0", "0")]),
    "0334": ("increasingTriplet", [("[1,2,3,4,5]", "True"), ("[5,4,3,2,1]", "False")]),
    "0338": ("countBits", [("2", "[0, 1, 1]"), ("5", "[0, 1, 1, 2, 1, 2]")]),
    "0367": ("isPerfectSquare", [("16", "True"), ("14", "False")]),
    "0377": ("combinationSum4", [("[1,2,3], 4", "7")]),
    "0383": ("canConstruct", [('\"a\", \"b\"', "False"), ('\"aa\", \"aab\"', "True")]),
    "0394": ("decodeString", [('\"3[a]2[bc]\"', "aaabcbc"), ('\"3[a2[c]]\"', "accaccacc")]),
    "0402": ("removeKdigits", [('\"1432219\", 3', "1219"), ('\"10200\", 1', "200"), ('\"10\", 2', "0")]),
    "0410": ("splitArray", [("[7,2,5,10,8], 2", "18")]),
    "0416": ("canPartition", [("[1,5,11,5]", "True"), ("[1,2,3,5]", "False")]),
    "0424": ("characterReplacement", [('\"ABAB\", 2', "4"), ('\"AABABBA\", 1', "4")]),
    "0435": ("eraseOverlapIntervals", [("[[1,2],[2,3],[3,4],[1,3]]", "1"), ("[[1,2],[1,2],[1,2]]", "2")]),
    "0438": ("findAnagrams", [('\"cbaebabacd\", \"abc\"', "[0, 6]"), ('\"abab\", \"ab\"', "[0, 1, 2]")]),
    "0441": ("arrangeCoins", [("5", "2"), ("8", "3")]),
    "0452": ("findMinArrowShots", [("[[10,16],[2,8],[1,6],[7,12]]", "2")]),
    "0456": ("find132pattern", [("[1,2,3,4]", "False"), ("[3,1,4,2]", "True")]),
    "0459": ("repeatedSubstringPattern", [('\"abab\"', "True"), ('\"aba\"', "False")]),
    "0494": ("findTargetSumWays", [("[1,1,1,1,1], 3", "5")]),
    "0496": ("nextGreaterElement", [("[4,1,2], [1,3,4,2]", "[-1, 3, -1]")]),
    "0516": ("longestPalindromeSubseq", [('\"bbbab\"', "4"), ('\"cbbd\"', "2")]),
    "0518": ("change", [("5, [1,2,5]", "4"), ("3, [2]", "0")]),
    "0523": ("checkSubarraySum", [("[23,2,4,6,7], 6", "True")]),
    "0525": ("findMaxLength", [("[0,1]", "2"), ("[0,1,0]", "2")]),
    "0540": ("singleNonDuplicate", [("[1,1,2,3,3,4,4,8,8]", "2")]),
    "0560": ("subarraySum", [("[1,1,1], 2", "2"), ("[1,2,3], 3", "2")]),
    "0567": ("checkInclusion", [('\"ab\", \"eidbaooo\"', "True"), ('\"ab\", \"eidboaoo\"', "False")]),
    "0647": ("countSubstrings", [('\"abc\"', "3"), ('\"aaa\"', "6")]),
    "0658": ("findClosestElements", [("[1,2,3,4,5], 4, 3", "[1, 2, 3, 4]")]),
    "0665": ("checkPossibility", [("[4,2,3]", "True"), ("[4,2,1]", "False")]),
    "0673": ("findNumberOfLIS", [("[1,3,5,4,7]", "2"), ("[2,2,2,2,2]", "5")]),
    "0678": ("checkValidString", [('\"()\"', "True"), ('\"(*)\"', "True"), ('\"(*))\"', "True")]),
    "0680": ("validPalindrome", [('\"aba\"', "True"), ('\"abca\"', "True"), ('\"abc\"', "False")]),
    "0682": ("calPoints", [('[\"5\",\"2\",\"C\",\"D\",\"+\"]', "30")]),
    "0724": ("pivotIndex", [("[1,7,3,6,5,6]", "3"), ("[1,2,3]", "-1")]),
    "0735": ("asteroidCollision", [("[5,10,-5]", "[5, 10]"), ("[8,-8]", "[]"), ("[10,2,-5]", "[10]")]),
    "0739": ("dailyTemperatures", [("[73,74,75,71,69,72,76,73]", "[1, 1, 4, 2, 1, 1, 0, 0]")]),
    "0740": ("deleteAndEarn", [("[3,4,2]", "6"), ("[2,2,3,3,3,4]", "9")]),
    "0746": ("minCostClimbingStairs", [("[10,15,20]", "15")]),
    "0763": ("partitionLabels", [('\"ababcbacadefegdehijhklij\"', "[9, 7, 8]")]),
    "0791": ("customSortString", [('\"cba\", \"abcd\"', "cbad")]),
    "0853": ("carFleet", [("12, [10,8,0,5,3], [2,4,1,1,3]", "3")]),
    "0875": ("minEatingSpeed", [("[3,6,7,11], 8", "4"), ("[30,11,23,4,20], 5", "30")]),
    "0881": ("numRescueBoats", [("[1,2], 3", "1"), ("[3,2,2,1], 3", "3")]),
    "0896": ("isMonotonic", [("[1,2,2,3]", "True"), ("[6,5,4,4]", "True"), ("[1,3,2]", "False")]),
    "0912": ("sortArray", [("[5,2,3,1]", "[1, 2, 3, 5]")]),
    "0918": ("maxSubarraySumCircular", [("[1,-2,3,-2]", "3"), ("[5,-3,5]", "10"), ("[-3,-2,-3]", "-2")]),
    "0930": ("numSubarraysWithSum", [("[1,0,1,0,1], 2", "4")]),
    "0946": ("validateStackSequences", [("[1,2,3,4,5], [4,5,3,2,1]", "True"), ("[1,2,3,4,5], [4,3,5,1,2]", "False")]),
    "0948": ("bagOfTokensScore", [("[100], 50", "0"), ("[200,100], 150", "1")]),
    "0977": ("sortedSquares", [("[-4,-1,0,3,10]", "[0, 1, 9, 16, 100]")]),
    "0978": ("maxTurbulenceSize", [("[9,4,2,10,7,8,8,1,9]", "5"), ("[100]", "1")]),
    "0997": ("findJudge", [("2, [[1,2]]", "2"), ("3, [[1,3],[2,3]]", "3"), ("3, [[1,3],[2,3],[3,1]]", "-1")]),
    "1011": ("shipWithinDays", [("[1,2,3,4,5,6,7,8,9,10], 5", "15")]),
    "1029": ("twoCitySchedCost", [("[[10,20],[30,200],[400,50],[30,20]]", "110")]),
    "1049": ("lastStoneWeightII", [("[2,7,4,1,8,1]", "1")]),
    "1137": ("tribonacci", [("4", "4"), ("25", "1389537")]),
    "1143": ("longestCommonSubsequence", [('\"abcde\", \"ace\"', "3"), ('\"abc\", \"def\"', "0")]),
    "1209": ("removeDuplicates", [('\"abcd\", 2', "abcd"), ('\"deeedbbcccbdaa\", 3', "aa")]),
    "1220": ("countVowelPermutation", [("1", "5"), ("2", "10"), ("5", "68")]),
    "1288": ("removeCoveredIntervals", [("[[1,4],[3,6],[2,8]]", "2")]),
    "1299": ("replaceElements", [("[17,18,5,4,6,1]", "[18, 6, 6, 6, 1, -1]")]),
    "1343": ("numOfSubarrays", [("[2,1,5,6,0,9,8], 3, 4", "3")]),
    "1423": ("maxScore", [("[1,2,3,4,5,6,1], 3", "12")]),
    "1456": ("maxVowels", [('\"abciiidef\", 3', "3"), ('\"aeiou\", 2', "2")]),
    "1464": ("maxProduct", [("[3,4,5,2]", "12"), ("[1,5,4,5]", "16")]),
    "1475": ("finalPrices", [("[8,4,6,2,3]", "[4, 2, 4, 2, 3]")]),
    "1523": ("countOdds", [("3, 7", "3"), ("8, 10", "1")]),
    "1572": ("diagonalSum", [("[[1,2,3],[4,5,6],[7,8,9]]", "25")]),
    "1658": ("minOperations", [("[1,1,4,2,3], 5", "2"), ("[5,6,7,8,9], 4", "-1")]),
    "1700": ("countStudents", [("[1,1,0,0], [0,1,0,1]", "0")]),
    "1750": ("minimumLength", [('\"ca\"', "2"), ('\"cabaabac\"', "0")]),
    "1800": ("maxAscendingSum", [("[10,20,30,5,10,50]", "65")]),
    "1822": ("arraySign", [("[-1,-2,-3,-4,3,2,1]", "1"), ("[1,5,0,2,-3]", "0"), ("[-1,1,-1,1,-1]", "-1")]),
    "1849": ("splitString", [('\"1234\"', "False"), ('\"050043\"', "True")]),
    "1899": ("mergeTriplets", [("[[2,5,3],[1,8,4],[1,7,5]], [2,7,5]", "True")]),
    "1963": ("minSwaps", [('\"][][\"', "1"), ('\"[][][]\"', "0")]),
    "1968": ("rearrangeArray", [("[3,1,-2,-5,2,-4]", "[3, -2, 1, -5, 2, -4]")]),
    "1984": ("minimumDifference", [("[90], 1", "0")]),
    "2235": ("sum", [("12, 5", "17"), ("-10, 4", "-6")]),
    "2300": ("successfulPairs", [("[5,1,3], [1,2,3,4,5], 7", "[4, 0, 3]")]),
    "2348": ("zeroFilledSubarray", [("[1,3,0,0,2,0,0,4]", "6"), ("[0,0,0,2,0,0]", "9")]),
    "2390": ("removeStars", [('\"leet**cod*e\"', "lecoe")]),
    "2405": ("minPartitions", [('\"abacbc\"', "3")]),
    "2483": ("bestClosingTime", [('\"YYNY\"', "2"), ('\"NNNNN\"', "0")]),
    "2554": ("maxCount", [("6, [1,6,5], 2", "3")]),
    "2616": ("minimizeMax", [("[10,1,2,7,1,3], 2", "1")]),
    "2864": ("maximumOddBinaryNumber", [('\"010\"', "001"), ('\"0101\"', "1001")]),
    "2971": ("largestPerimeter", [("[5,5,5]", "15"), ("[1,12,1,2,5,50,3]", "12")]),
}


def slug(fn):
    return fn.replace(".py","").replace("-","_")

def num(fn):
    return fn[:4]

def nice_name(fn):
    n = fn.replace(".py","")
    n = re.sub(r'^\d+-','',n)
    return n.replace("-"," ").title()

def read_file(path):
    with open(path) as f:
        return f.read()

def extract_and_convert(src):
    """Extract Solution class methods, convert to standalone functions."""
    lines = src.split("\n")
    
    # Collect everything outside class Solution as-is (helper functions, constants)
    outside = []
    in_class = False
    class_methods = []
    current_method = []
    
    for line in lines:
        # Skip imports and type hints
        if line.strip().startswith("import ") or line.strip().startswith("from "):
            continue
        if line.strip().startswith("#") and not in_class:
            continue
            
        if re.match(r'^class\s+Solution', line):
            in_class = True
            continue
        
        if in_class:
            # Detect end of class
            if line.strip() and not line[0].isspace() and not line.strip().startswith("#"):
                in_class = False
                if current_method:
                    class_methods.append(current_method)
                    current_method = []
                outside.append(line)
                continue
            
            # Inside class
            if re.match(r'    def\s+', line) or re.match(r'\tdef\s+', line):
                if current_method:
                    class_methods.append(current_method)
                current_method = [line]
            else:
                current_method.append(line)
        else:
            outside.append(line)
    
    if current_method:
        class_methods.append(current_method)
    
    # Convert class methods to standalone functions
    result = []
    for method_lines in class_methods:
        converted = []
        for line in method_lines:
            # Remove one level of indent
            if line.startswith("    "):
                line = line[4:]
            elif line.startswith("\t"):
                line = line[1:]
            # Remove self
            line = re.sub(r'\(self,\s*', '(', line)
            line = re.sub(r'\(self\)', '()', line)
            line = line.replace("self.", "")
            converted.append(line)
        result.extend(converted)
        result.append("")
    
    # Add outside code
    for line in outside:
        if line.strip():
            result.append(line)
    
    return "\n".join(result)

def to_sifr_types(text):
    """Convert Python type hints to Sifr."""
    t = text
    t = re.sub(r'List\[List\[List\[(\w+)\]\]\]', r'list[list[list[\1]]]', t)
    t = re.sub(r'List\[List\[(\w+)\]\]', r'list[list[\1]]', t)
    t = re.sub(r'List\[(\w+)\]', r'list[\1]', t)
    t = re.sub(r'Optional\[(\w+)\]', r'\1 | None', t)
    # Remove docstrings
    t = re.sub(r'"""[\s\S]*?"""', '', t)
    t = re.sub(r"'''[\s\S]*?'''", '', t)
    return t

def compile_sifr(sifr_path):
    """Compile a .sifr file and return (exit_code, stdout, stderr)."""
    try:
        result = subprocess.run(
            ["cargo", "run", "--release", "-p", "sifr", "--", "run", sifr_path],
            capture_output=True, text=True, timeout=30,
            cwd=WORKSPACE
        )
        return result.returncode, result.stdout.strip(), result.stderr.strip()
    except subprocess.TimeoutExpired:
        return -1, "", "TIMEOUT"
    except Exception as e:
        return -2, "", str(e)

def main():
    # First, build sifr
    print("Building sifr...")
    subprocess.run(["cargo", "build", "--release", "-p", "sifr"], cwd=WORKSPACE, capture_output=True)
    
    files = sorted(os.listdir(SRC))
    
    results = []
    
    for fn in files:
        if not fn.endswith(".py"):
            continue
        n = num(fn)
        if n in DONE:
            continue
        
        s = slug(fn)
        name = nice_name(fn)
        src = read_file(os.path.join(SRC, fn))
        
        # Categorize
        has_node = bool(re.search(r'ListNode|TreeNode|Optional\[', src))
        has_init = bool(re.search(r'__init__', src))
        
        if has_node:
            cat = "node"
        elif has_init:
            cat = "design"
        else:
            cat = "algo"
        
        # Check if we have test cases
        has_tests = n in TEST_CASES and len(TEST_CASES[n][1]) > 0
        
        # Convert
        body = extract_and_convert(src)
        sifr_body = to_sifr_types(body)
        
        # Clean up empty lines
        sifr_lines = sifr_body.split("\n")
        while sifr_lines and sifr_lines[0].strip() == "":
            sifr_lines.pop(0)
        while sifr_lines and sifr_lines[-1].strip() == "":
            sifr_lines.pop()
        sifr_body = "\n".join(sifr_lines)
        
        # Build Sifr file
        sifr_content = f"# LeetCode {int(n)}: {name}\n\n"
        sifr_content += sifr_body + "\n\n"
        
        if has_tests:
            func_name, tests = TEST_CASES[n]
            sifr_content += "def main():\n"
            for args, expected in tests:
                sifr_content += f"    print({func_name}({args}))\n"
            sifr_content += "\n"
            for args, expected in tests:
                sifr_expected = expected.replace("True", "true").replace("False", "false")
                sifr_content += f"# expect-stdout: {sifr_expected}\n"
        else:
            sifr_content += "def main():\n"
            sifr_content += '    print("no test cases")\n'
        
        sifr_content += "\n"
        
        # Build Python file
        py_content = f"# LeetCode {int(n)}: {name}\n# Python version\n\n"
        py_content += body + "\n\n"
        if has_tests:
            func_name, tests = TEST_CASES[n]
            py_content += "def main():\n"
            for args, expected in tests:
                py_content += f"    print({func_name}({args}))\n"
            py_content += '\nif __name__ == "__main__":\n    main()\n'
        else:
            py_content += 'def main():\n    print("no test cases")\n\nif __name__ == "__main__":\n    main()\n'
        
        # Write files
        sifr_path = os.path.join(DST, f"{s}.sifr")
        py_path = os.path.join(DST, f"{s}.py")
        
        with open(sifr_path, "w") as f:
            f.write(sifr_content)
        with open(py_path, "w") as f:
            f.write(py_content)
        
        # Compile Sifr
        exit_code, stdout, stderr = compile_sifr(sifr_path)
        
        # Determine result
        if exit_code == -1:
            status = "TIMEOUT"
            error = "Compilation or execution timed out"
        elif exit_code != 0 or "type error" in stderr or "parse error" in stderr or "error" in stderr.lower():
            status = "COMPILE_ERROR"
            # Extract first error
            error_lines = [l for l in stderr.split("\n") if "error" in l.lower() or "type error" in l.lower()]
            error = error_lines[0] if error_lines else stderr[:200]
        elif has_tests:
            # Check output
            func_name, tests = TEST_CASES[n]
            expected_lines = []
            for args, expected in tests:
                sifr_expected = expected.replace("True", "true").replace("False", "false")
                expected_lines.append(sifr_expected)
            expected_output = "\n".join(expected_lines)
            
            if stdout == expected_output:
                status = "PASS"
                error = ""
            else:
                status = "WRONG_OUTPUT"
                error = f"Expected: {expected_output[:100]}... Got: {stdout[:100]}..."
        else:
            if "no test cases" in stdout:
                status = "COMPILES_NO_TESTS"
                error = ""
            elif stdout:
                status = "COMPILES_NO_TESTS"
                error = ""
            else:
                status = "COMPILES_NO_TESTS"
                error = ""
        
        results.append({
            "num": n,
            "slug": s,
            "name": name,
            "category": cat,
            "has_tests": has_tests,
            "status": status,
            "error": error,
            "stdout": stdout[:200] if stdout else "",
        })
        
        # Print progress
        status_icon = {"PASS": "✓", "COMPILE_ERROR": "✗", "WRONG_OUTPUT": "≠", "TIMEOUT": "⏱", "COMPILES_NO_TESTS": "~"}.get(status, "?")
        print(f"  {status_icon} {n} {name}: {status}" + (f" — {error[:80]}" if error else ""))
    
    # Write results JSON
    with open(os.path.join(DST, "audit_results.json"), "w") as f:
        json.dump(results, f, indent=2)
    
    # Print summary
    print("\n" + "="*60)
    print("AUDIT SUMMARY")
    print("="*60)
    
    by_status = {}
    for r in results:
        by_status.setdefault(r["status"], []).append(r)
    
    for status in ["PASS", "COMPILES_NO_TESTS", "WRONG_OUTPUT", "COMPILE_ERROR", "TIMEOUT"]:
        if status in by_status:
            print(f"  {status}: {len(by_status[status])}")
    
    print(f"\n  Total: {len(results)}")
    
    # Error categorization
    if "COMPILE_ERROR" in by_status:
        error_types = {}
        for r in by_status["COMPILE_ERROR"]:
            err = r["error"]
            if "type mismatch" in err or "int | None" in err or "str | None" in err or "None |" in err:
                key = "safe_indexing_optional"
            elif "assignment target must be a simple name" in err:
                key = "subscript_assignment"
            elif "undefined variable" in err:
                key = "undefined_variable"
            elif "unsupported operand" in err:
                key = "unsupported_operand"
            elif "parse error" in err or "unexpected" in err:
                key = "parse_error"
            elif "not supported between" in err:
                key = "comparison_optional"
            elif "has no method" in err or "no method" in err:
                key = "missing_method"
            elif "cannot compare" in err:
                key = "comparison_error"
            elif "not callable" in err:
                key = "not_callable"
            elif "too many" in err or "too few" in err:
                key = "argument_count"
            else:
                key = "other"
            error_types.setdefault(key, []).append(r["num"])
        
        print("\n  Error Categories:")
        for key, nums in sorted(error_types.items(), key=lambda x: -len(x[1])):
            print(f"    {key}: {len(nums)} problems")
    
    print(f"\n  Results written to {DST}/audit_results.json")


if __name__ == "__main__":
    main()
