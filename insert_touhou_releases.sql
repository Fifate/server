-- 插入东方Project相关的发行版数据
INSERT INTO "release" (
  "title", 
  "release_type", 
  "release_date", 
  "release_date_precision",
  "recording_date_start", 
  "recording_date_start_precision", 
  "recording_date_end", 
  "recording_date_end_precision"
) VALUES
-- 東方妖々夢 ～ Perfect Cherry Blossom (2003)
('東方妖々夢 ～ Perfect Cherry Blossom', 'Album', '2003-08-17', 'Day', '2003-01-01', 'Month', '2003-07-31', 'Month'),

-- 東方永夜抄 ～ Imperishable Night (2004)  
('東方永夜抄 ～ Imperishable Night', 'Album', '2004-08-15', 'Day', '2004-02-01', 'Month', '2004-07-31', 'Month'),

-- 東方風神録 ～ Mountain of Faith (2007)
('東方風神録 ～ Mountain of Faith', 'Album', '2007-08-17', 'Day', '2007-03-01', 'Month', '2007-07-31', 'Month'),

-- Bad Apple!! feat. nomico - 单曲 (2007)
('Bad Apple!! feat. nomico', 'Single', '2007-08-17', 'Day', '2007-06-01', 'Month', '2007-07-15', 'Month'),

-- 東方地霊殿 ～ Subterranean Animism (2008)
('東方地霊殿 ～ Subterranean Animism', 'Album', '2008-08-16', 'Day', '2008-02-01', 'Month', '2008-07-31', 'Month'),

-- 東方星蓮船 ～ Undefined Fantastic Object (2009)
('東方星蓮船 ～ Undefined Fantastic Object', 'Album', '2009-08-15', 'Day', '2009-01-01', 'Year', '2009-07-31', 'Month'),

-- 東方神霊廟 ～ Ten Desires (2011)
('東方神霊廟 ～ Ten Desires', 'Album', '2011-08-13', 'Day', '2011-02-01', 'Month', '2011-07-31', 'Month'),

-- 東方輝針城 ～ Double Dealing Character (2013)
('東方輝針城 ～ Double Dealing Character', 'Album', '2013-08-12', 'Day', '2013-03-01', 'Month', '2013-07-31', 'Month'),

-- 東方紺珠伝 ～ Legacy of Lunatic Kingdom (2015)
('東方紺珠伝 ～ Legacy of Lunatic Kingdom', 'Album', '2015-08-14', 'Day', '2015-02-01', 'Month', '2015-07-31', 'Month'),

-- 東方虹龍洞 ～ Unconnected Marketeers (2021)
('東方虹龍洞 ～ Unconnected Marketeers', 'Album', '2021-05-04', 'Day', '2021-01-01', 'Year', '2021-04-30', 'Month');