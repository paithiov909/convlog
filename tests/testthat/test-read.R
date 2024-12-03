test_that("read_tenhou6 works", {
  dir <- system.file("testdata/", package = "convlog")
  files <- list.files(dir, pattern = "*.json$", full.names = TRUE)
  out <- read_tenhou6(files)
  expect_equal(names(out), c("game_info", "round_info", "paifu"))
  expect_true(inherits(out[["game_info"]], "tbl_df"))
  expect_true(inherits(out[["round_info"]], "tbl_df"))
  expect_true(inherits(out[["paifu"]], "tbl_df"))
})

test_that("read_mjlog works", {
  dir <- system.file("mjlog/", package = "convlog")
  files <- list.files(dir, pattern = "*.mjlog$", full.names = TRUE)
  out <- read_mjlog(files)
  expect_equal(names(out), c("game_info", "round_info", "paifu"))
  expect_true(inherits(out[["game_info"]], "tbl_df"))
  expect_true(inherits(out[["round_info"]], "tbl_df"))
  expect_true(inherits(out[["paifu"]], "tbl_df"))
})
