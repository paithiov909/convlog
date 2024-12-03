test_that("read_remote_mjlog and read_mjlog returns almost the same result", {
  skip_on_cran()
  skip_if_offline()

  logid <- "2010091009gm-00a9-0000-83af2648"

  expected <- read_remote_mjlog(logid)
  actual <- read_mjlog(
    system.file(paste0("mjlog/", logid, "&tw=2.mjlog"), package = "convlog")
  )

  expect_equal(expected[["game_info"]], actual[["game_info"]])
  expect_equal(
    dplyr::select(expected[["round_info"]], !c("tehais")),
    dplyr::select(actual[["round_info"]], !c("tehais"))
  )
  expect_equal(
    dplyr::filter(expected[["paifu"]], type != "reach_accepted") |>
      dplyr::select(!c("round_id", "tsumogiri", "ura_markers")),
    dplyr::filter(actual[["paifu"]], type != "reach_accepted") |>
      dplyr::select(!c("round_id", "tsumogiri", "ura_markers"))
  )
})
