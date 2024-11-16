
<!-- README.md is generated from README.Rmd. Please edit that file -->

# convlog

<!-- badges: start -->
<!-- badges: end -->

convlog offers wrappers for the ‘convlog’ Rust crate from
[Equim-chan/mjai-reviewer](https://github.com/Equim-chan/mjai-reviewer)
that can directly read mahjong logs from ‘tenhou.net/6’ format into
tibbles.

## Installation

To install from source package, the Rust toolchain is required.

``` r
# install.packages("pak")
pak::pak("paithiov909/convlog")
```

## Example

``` r
library(convlog)

read_tenhou6(system.file("testdata/output_log.example.json", package = "convlog"))
#> $game_info
#> # A tibble: 1 × 4
#>   game_id names        qijia aka  
#>     <int> <named list> <int> <lgl>
#> 1       1 <chr [4]>        0 TRUE 
#> 
#> $round_info
#> # A tibble: 10 × 10
#>    game_id round_id bakaze dora_marker kyoku honba kyotaku   oya scores tehais  
#>      <int>    <int> <chr>  <chr>       <int> <int>   <int> <int> <list> <list>  
#>  1       1        1 E      7p              1     0       0     0 <int>  <chr[…]>
#>  2       1        2 E      8m              2     0       0     1 <int>  <chr[…]>
#>  3       1        3 E      N               3     0       0     2 <int>  <chr[…]>
#>  4       1        4 E      6m              4     0       0     3 <int>  <chr[…]>
#>  5       1        5 E      6p              4     1       0     3 <int>  <chr[…]>
#>  6       1        6 S      6m              1     0       0     0 <int>  <chr[…]>
#>  7       1        7 S      E               1     1       0     0 <int>  <chr[…]>
#>  8       1        8 S      E               2     0       0     1 <int>  <chr[…]>
#>  9       1        9 S      1s              3     0       0     2 <int>  <chr[…]>
#> 10       1       10 S      5s              4     1       1     3 <int>  <chr[…]>
#> 
#> $paifu
#> # A tibble: 1,031 × 12
#>    game_id round_id event_id type  actor target pai   tsumogiri consumed
#>      <int>    <int>    <int> <chr> <int>  <int> <chr> <lgl>     <list>  
#>  1       1        1        1 tsumo     0     NA 3s    NA        <NULL>  
#>  2       1        1        2 dahai     0     NA C     FALSE     <NULL>  
#>  3       1        1        3 tsumo     1     NA 9m    NA        <NULL>  
#>  4       1        1        4 dahai     1     NA N     FALSE     <NULL>  
#>  5       1        1        5 tsumo     2     NA 5p    NA        <NULL>  
#>  6       1        1        6 dahai     2     NA E     FALSE     <NULL>  
#>  7       1        1        7 tsumo     3     NA S     NA        <NULL>  
#>  8       1        1        8 dahai     3     NA C     FALSE     <NULL>  
#>  9       1        1        9 tsumo     0     NA 6p    NA        <NULL>  
#> 10       1        1       10 dahai     0     NA 7m    FALSE     <NULL>  
#> # ℹ 1,021 more rows
#> # ℹ 3 more variables: dora_marker <chr>, deltas <list>, ura_markers <list>
```
