package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"strings"
)

// sanitizeCharForIdentifier はルーン文字をRustの識別子として安全な文字列に変換します。
func sanitizeCharForIdentifier(r rune) string {
	return fmt.Sprintf("HEX_%02X", int(r))
}

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintf(os.Stderr, "使用法: %s <入力ファイル> <出力ファイル>\n", os.Args[0])
		os.Exit(1)
	}
	inputFile := os.Args[1]
	outputFile := os.Args[2]

	in, err := os.Open(inputFile)
	if err != nil {
		log.Fatalf("入力ファイル '%s' を開けませんでした: %v", inputFile, err)
	}
	defer in.Close()

	out, err := os.Create(outputFile)
	if err != nil {
		log.Fatalf("出力ファイル '%s' を作成できませんでした: %v", outputFile, err)
	}
	defer out.Close()

	writer := bufio.NewWriter(out)
	scanner := bufio.NewScanner(in)

	var currentChar rune // 現在処理中の文字 (0 はヘッダー待ち)
	var bitmapLines []string
	var expectedBitmapLines = 16
	inputLineNum := 0

	for scanner.Scan() {
		inputLineNum++
		line := scanner.Text()

		if strings.TrimSpace(line) == "" { // 空行はブロックの区切り
			if currentChar != 0 { // ヘッダーが読み込まれていたが、ビットマップが不完全だった場合
				if len(bitmapLines) > 0 && len(bitmapLines) < expectedBitmapLines {
					log.Printf("警告: 文字 '%c' (0x%X) のビットマップデータが%d行しかなく不完全です（入力ファイル %d行目付近）。この文字の出力はスキップされます。", currentChar, currentChar, len(bitmapLines), inputLineNum)
				}
			}
			currentChar = 0 // 状態リセット
			bitmapLines = []string{}
			continue
		}

		// ヘッダー行の解析 (例: 0x41 'A')
		var parsedHexVal int
		var parsedChar rune
		// fmt.Sscanf は ' ' (スペース) は正しく読み取れますが、 ''' (アポストロフィ) のような
		// 特殊なケースでは期待通りに動作しない可能性があります。
		// ここではユーザー指定のフォーマット例に準拠します。
		log.Printf("line: %s", line)
		numScanned, scanErr := fmt.Sscanf(line, "0x%x '%c'", &parsedHexVal, &parsedChar)

		if scanErr != nil && numScanned == 1 {
			_, scanErr = fmt.Sscanf(line, "0x%x", &parsedHexVal)
			parsedChar = rune(parsedHexVal)
		}

		log.Printf("numScanned: %d, scanErr: %v", numScanned, scanErr)

		if scanErr == nil { // ヘッダー行として解釈成功
			if currentChar != 0 && len(bitmapLines) > 0 { // 前の文字の処理が完了する前に新しいヘッダー
				log.Printf("警告: 文字 '%c' (0x%X) の処理中に新しいヘッダー行 (0x%X) が見つかりました（入力ファイル %d行目）。前の文字のデータは破棄されます。", currentChar, currentChar, parsedHexVal, inputLineNum)
			}
			currentChar = parsedChar
			bitmapLines = []string{} // 新しい文字のためにクリア

			identifier := sanitizeCharForIdentifier(currentChar)
			_, err = writer.WriteString(fmt.Sprintf("const K_FONT_%s: [u8; %d] = [\n", identifier, expectedBitmapLines))
			if err != nil {
				log.Fatalf("出力ファイルへの書き込みエラー（ヘッダー）: %v", err)
			}

		} else { // ヘッダーが既に読み込まれていれば、ビットマップ行とみなす
			if len(line) != 8 {
				log.Printf("警告: 文字 '%c' (0x%X) のビットマップ行（入力ファイル %d行目: \"%s\"）が8文字ではありません。この文字の出力はスキップされます。", currentChar, currentChar, inputLineNum, line)
				currentChar = 0 // この文字の処理を中断
				bitmapLines = []string{}
				continue
			}
			bitmapLines = append(bitmapLines, line)

			if len(bitmapLines) == expectedBitmapLines { // 16行のビットマップが集まった
				validBlock := true
				for _, bitmapRow := range bitmapLines {
					binaryString := ""
					commentRepresentation := ""
					if len(bitmapRow) != 8 { //念のため再チェック（通常は上のチェックで防がれる）
						log.Printf("エラー: 文字 '%c' (0x%X) のビットマップ行（\"%s\"）の長さが不正です。この文字の出力はスキップされます。", currentChar, currentChar, bitmapRow)
						currentChar = 0
						validBlock = false
						break
					}

					for _, pixelRune := range bitmapRow {
						switch pixelRune {
						case '@':
							binaryString += "1"
							commentRepresentation += "*"
						case '.':
							binaryString += "0"
							commentRepresentation += " "
						default:
							log.Printf("エラー: 文字 '%c' (0x%X) のビットマップ行（入力ファイル %d行目: \"%s\"）に不正な文字 '%c' が含まれています。この文字の出力はスキップされます。", currentChar, currentChar, inputLineNum, bitmapRow, pixelRune)
							currentChar = 0 // この文字の処理を中断
							validBlock = false
							break
						}
					}
					if !validBlock {
						break
					} // ビットマップ行のループを抜ける

					_, err = writer.WriteString(fmt.Sprintf("    0b%s, // %s\n", binaryString, commentRepresentation))
					if err != nil {
						log.Fatalf("出力ファイルへの書き込みエラー（データ行）: %v", err)
					}
				}

				if validBlock { // エラーで中断されていなければ配列を閉じる
					_, err = writer.WriteString("];\n\n") // 配列の終了と空行
					if err != nil {
						log.Fatalf("出力ファイルへの書き込みエラー（フッター）: %v", err)
					}
				}

				// 次の文字のために状態をリセット
				currentChar = 0
				bitmapLines = []string{}
			}
		}
	}

	if err := scanner.Err(); err != nil {
		log.Fatalf("入力ファイルの読み込み中にエラーが発生しました: %v", err)
	}

	// ファイル末尾で、まだ処理されていないビットマップデータが残っている場合の最終チェック
	if currentChar != 0 && len(bitmapLines) > 0 && len(bitmapLines) < expectedBitmapLines {
		log.Printf("警告: ファイル終端ですが、文字 '%c' (0x%X) のビットマップデータが%d行しかなく不完全です。この文字は出力されませんでした。", currentChar, currentChar, len(bitmapLines))
	}

	if err := writer.Flush(); err != nil {
		log.Fatalf("出力ファイルへの最終フラッシュエラー: %v", err)
	}
	fmt.Printf("Rustフォントファイル '%s' が正常に生成されました。\n", outputFile)
}
