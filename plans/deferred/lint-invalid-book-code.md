# Discussing (≈approved, filing): lint rule for invalid `\id` book code

Filed 2026-07-23. Low-risk, almost certainly approved — logging for proper filing.

**State:** onion already validates book codes at lex time — `src/lexer.rs:519
is_valid_book_code` is a `matches!` over the canonical set (109 codes, all 3-ASCII),
and `BookCode` tokens carry `is_valid: bool` (invalid → `BookId(???)`). The *data
exists*; lexing computes it.

**Gap:** there is no **lint rule** that surfaces `is_valid == false` as a finding.
The flag is computed but never reported to the user.

**Action:**
1. Add a `LintCode` (e.g. `InvalidBookCode`) that emits on a `BookCode` token with
   `is_valid == false`. Trivial — the evidence is already on the token.
2. Audit the `is_valid_book_code` `matches!` set against the USFM 3.1 books table
   (<https://docs.usfm.bible/usfm/3.1/doc/books.html>): 0-padded 1-87, the a4/a5/a6
   apocrypha codes, and 94-100. All `\id` codes are exactly 3 ASCII by spec; confirm
   completeness (esp. DC/apocrypha) and vendor/refresh if stale. Concordance/extra
   material (out of scope) still occupies 3-ASCII codes for spec sake.

Deliberate-behavior-change → its own oracle bless when implemented.



Table from https://docs.usfm.bible/usfm/latest/doc/books.html

Number,Identifier,English Name,Alternate Name / Notes
01,GEN,Genesis,‘1 Moses’ in some Bibles
02,EXO,Exodus,‘2 Moses’ in some Bibles
03,LEV,Leviticus,‘3 Moses’ in some Bibles
04,NUM,Numbers,‘4 Moses’ in some Bibles
05,DEU,Deuteronomy,‘5 Moses’ in some Bibles
06,JOS,Joshua,
07,JDG,Judges,
08,RUT,Ruth,
09,1SA,1 Samuel,1 Kings or Kingdoms in Orthodox Bibles; do not confuse with ISA (Isaiah)
10,2SA,2 Samuel,2 Kings or Kingdoms in Orthodox Bibles
11,1KI,1 Kings,3 Kings or Kingdoms in Orthodox Bibles
12,2KI,2 Kings,4 Kings or Kingdoms in Orthodox Bibles
13,1CH,1 Chronicles,1 Paralipomenon in Orthodox Bibles
14,2CH,2 Chronicles,2 Paralipomenon in Orthodox Bibles
15,EZR,Ezra,Hebrew Ezra (1 Ezra / 1 Esdras); also used for Ezra-Nehemiah combined
16,NEH,Nehemiah,Appended to Ezra; 2 Esdras in Vulgate
17,EST,Esther (Hebrew),Hebrew Esther (use ESG for Greek LXX Esther)
18,JOB,Job,
19,PSA,Psalms,"150 (Hebrew), 151 (Orthodox), 155 (West Syriac). Use PS2 for Psalm 151, PS3 for 152–155"
20,PRO,Proverbs,31 chapters (24 in Ethiopian Bible)
21,ECC,Ecclesiastes,Qoheleth in Catholic Bibles; use SIR for Ecclesiasticus
22,SNG,Song of Songs,Song of Solomon / Canticle of Canticles
23,ISA,Isaiah,Do not confuse with 1SA (1 Samuel)
24,JER,Jeremiah,Book of Jeremiah; use LJE for Letter of Jeremiah
25,LAM,Lamentations,Lamentations of Jeremiah
26,EZK,Ezekiel,
27,DAN,Daniel (Hebrew),Hebrew Daniel (use DAG for Greek LXX Daniel)
28,HOS,Hosea,
29,JOL,Joel,
30,AMO,Amos,
31,OBA,Obadiah,
32,JON,Jonah,Do not confuse with JHN (John)
33,MIC,Micah,
34,NAM,Nahum,
35,HAB,Habakkuk,
36,ZEP,Zephaniah,
37,HAG,Haggai,
38,ZEC,Zechariah,
39,MAL,Malachi,
41,MAT,Matthew,Gospel according to Matthew
42,MRK,Mark,Gospel according to Mark
43,LUK,Luke,Gospel according to Luke
44,JHN,John,Gospel according to John
45,ACT,Acts,Acts of the Apostles
46,ROM,Romans,Letter of Paul to the Romans
47,1CO,1 Corinthians,First Letter of Paul to the Corinthians
48,2CO,2 Corinthians,Second Letter of Paul to the Corinthians
49,GAL,Galatians,Letter of Paul to the Galatians
50,EPH,Ephesians,Letter of Paul to the Ephesians
51,PHP,Philippians,Letter of Paul to the Philippians
52,COL,Colossians,Letter of Paul to the Colossians
53,1TH,1 Thessalonians,First Letter of Paul to the Thessalonians
54,2TH,2 Thessalonians,Second Letter of Paul to the Thessalonians
55,1TI,1 Timothy,First Letter of Paul to Timothy
56,2TI,2 Timothy,Second Letter of Paul to Timothy
57,TIT,Titus,Letter of Paul to Titus
58,PHM,Philemon,Letter of Paul to Philemon
59,HEB,Hebrews,Letter to the Hebrews
60,JAS,James,Letter of James
61,1PE,1 Peter,First Letter of Peter
62,2PE,2 Peter,Second Letter of Peter
63,1JN,1 John,First Letter of John
64,2JN,2 John,Second Letter of John
65,3JN,3 John,Third Letter of John
66,JUD,Jude,Letter of Jude; do not confuse with JDG or JDT
67,REV,Revelation,Revelation to John; called Apocalypse in Catholic Bibles
68,TOB,Tobit,Deuterocanonical / Apocrypha
69,JDT,Judith,Deuterocanonical / Apocrypha
70,ESG,Esther Greek,Greek additions to Esther
71,WIS,Wisdom of Solomon,Deuterocanonical / Apocrypha
72,SIR,Sirach,Ecclesiasticus or Jesus son of Sirach
73,BAR,Baruch,"5 ch. (Orthodox), 6 ch. (Catholic, incl. LJE); 1 Baruch in Syriac"
74,LJE,Letter of Jeremiah,Sometimes in Baruch; ‘Rest of Jeremiah’ in Ethiopia
75,S3Y,Song of 3 Young Men,Includes Prayer of Azariah; sometimes in Greek Daniel
76,SUS,Susanna,Sometimes included in Greek Daniel
77,BEL,Bel and the Dragon,Sometimes included in Greek Daniel; ‘Rest of Daniel’ in Ethiopia
78,1MA,1 Maccabees,‘3 Maccabees’ in some traditions
79,2MA,2 Maccabees,‘1 Maccabees’ in some traditions
80,3MA,3 Maccabees,‘2 Maccabees’ in some traditions
81,4MA,4 Maccabees,Appendix to Greek Bible and Georgian Bible
82,1ES,1 Esdras (Greek),"9 ch. Greek Ezra in LXX; 2 Esdras (Russian), 3 Esdras (Vulgate)"
83,2ES,2 Esdras (Latin),"16 ch. Latin Esdras; 3 Esdras (Russian), 4 Esdras (Vulgate)"
84,MAN,Prayer of Manasseh,Appended to 2 Chronicles in Orthodox Bibles
85,PS2,Psalm 151,Additional Psalm in Septuagint / Orthodox Bibles
86,ODA,Odae/Odes,Septuagint collection; content varies by tradition
87,PSS,Psalms of Solomon,Septuagint book; not in modern Bibles
A4,EZA,Ezra Apocalypse,"12 ch. Apocalypse; 3 Ezra (Armenian), Ezra Shealtiel (Ethiopian)"
A5,5EZ,5 Ezra,2 ch. Latin preface to Ezra Apocalypse
A6,6EZ,6 Ezra,2 ch. Latin conclusion to Ezra Apocalypse
B2,DAG,Daniel Greek,14 ch. LXX version including Greek additions
B3,PS3,Psalms 152-155,Additional Psalms found in West Syriac manuscripts
B4,2BA,2 Baruch (Apocalypse),Apocalypse of Baruch in Syriac Bibles
B5,LBA,Letter of Baruch,Appended to or separate from 2 Baruch
B6,JUB,Jubilees,Ancient Hebrew book in Ethiopian Bible
B7,ENO,Enoch,1 Enoch; ancient Hebrew book in Ethiopian Bible
B8,1MQ,1 Meqabyan/Mekabis,Book of Mekabis of Benjamin (Ethiopian)
B9,2MQ,2 Meqabyan/Mekabis,Book of Mekabis of Moab (Ethiopian)
C0,3MQ,3 Meqabyan/Mekabis,Book of Meqabyan (Ethiopian)
C1,REP,Reproof,Proverbs part 2 (Ethiopian)
C2,4BA,4 Baruch,Paralipomenon of Jeremiah (Ethiopian)
C3,LAO,Letter to Laodiceans,Latin Vulgate book / medieval Catholic translations
A0,FRT,Front Matter,"Title page, prefatory text, etc."
A1,BAK,Back Matter,"Appendices, indexes, back cover text"
A2,OTH,Other Matter,Non-scriptural supplemental content
A7,INT,Introduction Matter,Book introductions
A8,CNC,Concordance,Concordance section
A9,GLO,Glossary/Wordlist,Glossary or dictionary
B0,TDX,Topical Index,Index of topics
B1,NDX,Names Index,Index of proper names
94,XXA,Extra material,Supplementary content container
95,XXB,Extra material,Supplementary content container
96,XXC,Extra material,Supplementary content container
97,XXD,Extra material,Supplementary content container
98,XXE,Extra material,Supplementary content container
99,XXF,Extra material,Supplementary content container
100,XXG,Extra material,Supplementary content container
