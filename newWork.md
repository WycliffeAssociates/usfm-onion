Date: March 05, 2026

We're about to undertake probably several high-level passes of work.. 

Overall end goal for this session: 
1. Capture the core vision and guiding pricinples for why we are creating this repo
2. Capture the current state of this repo, and update current.md.  We need a current.md that meshes with vision.md
3. For what we already implement, ensure spec compliance by comparing to the official docs first and foremost, with refercne implementations second: We'll likely need to revise what's existing
4. Ensure rust best practices and idiomatic rust is followed. Performance is nice, but correctness and maintainability are more important. We'll get performance after spec compliance is there. 

## repos_to_compare
- This is not our work. These are existing US F importers out there that try to tackle the problem in different ways. I'll be asking you to compare our work to them in a minute. 
- tcdocs-main - the officail spec. The absolute reference to abide by.  If repos implement inconsistent, this must when
- usfmtc-main - a python implementation of the spec, intended to be the reference on the docs. Of existing code, this is the most likely for getting spec compliant handling of usx and usj 
- usfm3-main - a recent rust implmentation that might give some idea of any rsut patterns to implement?
- usfm-grammar -> not likely the most useful, but included here in case. 