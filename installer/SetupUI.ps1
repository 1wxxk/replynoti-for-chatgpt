param()
$ErrorActionPreference='Stop'
Add-Type -AssemblyName PresentationFramework,PresentationCore,WindowsBase
[System.Windows.Media.RenderOptions]::ProcessRenderMode=[System.Windows.Interop.RenderMode]::SoftwareOnly
$root=Join-Path $env:LOCALAPPDATA 'ReplyNoti'
$upgrade=Test-Path (Join-Path $root '.replynoti-install')
$run=(Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue).ReplyNoti
$ko=[Globalization.CultureInfo]::CurrentUICulture.TwoLetterISOLanguageName -eq 'ko'
$t=if($ko){
 @{Title='ReplyNoti 설치';Step='설치 준비';Intro='답변이 준비되면, 알려드릴게요.';Options='설치 옵션';Startup='Windows 시작 시 실행';StartupDesc='로그인하면 자동으로 시작합니다.';Preview='답변 미리보기';PreviewDesc='잠금 화면 및 알림 기록에 답변이 표시될 수 있습니다.';Launch='설치 완료 후 실행';LaunchDesc='설치 후 알림 감지를 시작합니다.';Location='설치 위치';LocationDesc='현재 사용자 전용 설치 · 관리자 권한 불필요';Keep='업데이트 시 기존 설정과 사용자 지정 리소스를 유지합니다.';Install='설치';Update='업데이트';Cancel='취소';Close='닫기';Working='설치 중…';Done='설치가 완료되었습니다.';Failed='설치하지 못했습니다.';Retry='다시 시도';Unofficial='비공식 Windows 도우미';Privacy='별도 계정이나 API 키 없이 이 PC에서 동작합니다.';Hint='설치가 끝날 때까지 창을 닫지 마세요.';ErrorHint='다른 버전이 등록되어 있다면 먼저 제거하거나 등록을 해제하세요.'}
}else{
 @{Title='ReplyNoti Setup';Step='Ready to install';Intro='Know when your answer is ready.';Options='Installation options';Startup='Start with Windows';StartupDesc='Launch automatically when you sign in.';Preview='Reply preview';PreviewDesc='Replies may appear on your lock screen or in notification history.';Launch='Start after installation';LaunchDesc='Begin monitoring when setup is complete.';Location='Install location';LocationDesc='Current user only · No administrator access';Keep='Updates preserve existing preferences and custom assets.';Install='Install';Update='Update';Cancel='Cancel';Close='Close';Working='Installing…';Done='Installation complete.';Failed='Setup could not finish.';Retry='Try again';Unofficial='Independent Windows companion';Privacy='Runs on this PC without an extra account or API key.';Hint='Keep this window open until setup finishes.';ErrorHint='Remove or unregister another installed copy first.'}
}
$dark=(Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize' -ErrorAction SilentlyContinue).AppsUseLightTheme -eq 0
if($upgrade -and (Test-Path (Join-Path $root 'config.toml'))){
 $cfg=Get-Content (Join-Path $root 'config.toml') -Raw
 if($cfg -match '(?m)^theme\s*=\s*"dark"'){$dark=$true}
 elseif($cfg -match '(?m)^theme\s*=\s*"light"'){$dark=$false}
}
$xaml=@'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
 Title="ReplyNoti Setup" Width="740" SizeToContent="Height" WindowStartupLocation="CenterScreen" ResizeMode="NoResize"
 WindowStyle="None" AllowsTransparency="True" Background="Transparent" FontFamily="Segoe UI, Malgun Gothic" FontSize="14">
 <Window.Resources>
  <SolidColorBrush x:Key="Bg" Color="#191919"/><SolidColorBrush x:Key="Panel" Color="#202020"/>
  <SolidColorBrush x:Key="Card" Color="#222222"/><SolidColorBrush x:Key="Input" Color="#2C2C2C"/>
  <SolidColorBrush x:Key="Text" Color="#E7E7E7"/><SolidColorBrush x:Key="Muted" Color="#B0B0B0"/>
  <SolidColorBrush x:Key="Line" Color="#373737"/><SolidColorBrush x:Key="Accent" Color="#3265E6"/>
  <SolidColorBrush x:Key="AccentText" Color="#83A5FF"/>
  <Style TargetType="TextBlock"><Setter Property="Foreground" Value="{DynamicResource Text}"/><Setter Property="TextWrapping" Value="Wrap"/></Style>
  <Style TargetType="Button">
   <Setter Property="Foreground" Value="{DynamicResource Text}"/><Setter Property="Background" Value="{DynamicResource Input}"/><Setter Property="BorderBrush" Value="{DynamicResource Line}"/><Setter Property="BorderThickness" Value="1"/><Setter Property="Padding" Value="16,10"/><Setter Property="MinHeight" Value="42"/><Setter Property="Cursor" Value="Hand"/>
   <Setter Property="Template"><Setter.Value><ControlTemplate TargetType="Button">
    <Border x:Name="B" Background="{TemplateBinding Background}" BorderBrush="{TemplateBinding BorderBrush}" BorderThickness="{TemplateBinding BorderThickness}" CornerRadius="8" Padding="{TemplateBinding Padding}">
      <ContentPresenter HorizontalAlignment="Center" VerticalAlignment="Center" RecognizesAccessKey="True"/>
    </Border><ControlTemplate.Triggers>
      <Trigger Property="IsMouseOver" Value="True"><Setter TargetName="B" Property="Opacity" Value=".85"/></Trigger>
      <Trigger Property="IsKeyboardFocused" Value="True"><Setter TargetName="B" Property="BorderBrush" Value="{DynamicResource AccentText}"/><Setter TargetName="B" Property="BorderThickness" Value="2"/></Trigger>
      <Trigger Property="IsEnabled" Value="False"><Setter TargetName="B" Property="Opacity" Value=".45"/></Trigger>
    </ControlTemplate.Triggers></ControlTemplate></Setter.Value></Setter>
  </Style>
  <Style x:Key="Primary" TargetType="Button" BasedOn="{StaticResource {x:Type Button}}"><Setter Property="Foreground" Value="White"/><Setter Property="Background" Value="{DynamicResource Accent}"/><Setter Property="BorderBrush" Value="{DynamicResource Accent}"/><Setter Property="FontWeight" Value="SemiBold"/><Setter Property="MinWidth" Value="120"/></Style>
  <Style TargetType="CheckBox">
   <Setter Property="Foreground" Value="{DynamicResource Text}"/><Setter Property="VerticalContentAlignment" Value="Center"/><Setter Property="Cursor" Value="Hand"/>
   <Setter Property="Template"><Setter.Value><ControlTemplate TargetType="CheckBox">
    <Grid MinHeight="44" Background="Transparent"><Grid.ColumnDefinitions><ColumnDefinition Width="*"/><ColumnDefinition Width="44"/></Grid.ColumnDefinitions>
     <ContentPresenter Grid.Column="0" VerticalAlignment="Center" RecognizesAccessKey="True" Margin="0,0,12,0"/>
     <Border x:Name="Track" Grid.Column="1" Width="42" Height="24" CornerRadius="12" BorderBrush="{DynamicResource Line}" BorderThickness="1" Background="{DynamicResource Input}" VerticalAlignment="Center">
      <Ellipse x:Name="Thumb" Fill="{DynamicResource Muted}" Width="16" Height="16" HorizontalAlignment="Left" Margin="3,0" VerticalAlignment="Center"/>
     </Border>
    </Grid><ControlTemplate.Triggers>
     <Trigger Property="IsChecked" Value="True"><Setter TargetName="Track" Property="Background" Value="{DynamicResource Accent}"/><Setter TargetName="Track" Property="BorderBrush" Value="{DynamicResource Accent}"/><Setter TargetName="Thumb" Property="HorizontalAlignment" Value="Right"/><Setter TargetName="Thumb" Property="Fill" Value="White"/></Trigger>
     <Trigger Property="IsKeyboardFocused" Value="True"><Setter TargetName="Track" Property="BorderBrush" Value="{DynamicResource AccentText}"/><Setter TargetName="Track" Property="BorderThickness" Value="2"/></Trigger>
     <Trigger Property="IsEnabled" Value="False"><Setter Property="Opacity" Value=".5"/></Trigger>
    </ControlTemplate.Triggers></ControlTemplate></Setter.Value></Setter>
  </Style>
 </Window.Resources>
 <Border CornerRadius="12" Background="{DynamicResource Bg}" BorderBrush="{DynamicResource Line}" BorderThickness="1" ClipToBounds="True">
  <Grid>
   <Grid.RowDefinitions><RowDefinition Height="48"/><RowDefinition Height="Auto"/><RowDefinition Height="Auto"/></Grid.RowDefinitions>
   <Border Grid.Row="0" Background="{DynamicResource Panel}" BorderBrush="{DynamicResource Line}" BorderThickness="0,0,0,1" CornerRadius="11,11,0,0">
    <Grid Margin="14,0"><Grid.ColumnDefinitions><ColumnDefinition Width="*"/><ColumnDefinition Width="Auto"/></Grid.ColumnDefinitions>
     <StackPanel x:Name="TitleBar" Orientation="Horizontal" VerticalAlignment="Stretch" Background="Transparent"><Image x:Name="SmallIcon" Width="23" Height="23" Margin="0,0,9,0" RenderOptions.BitmapScalingMode="HighQuality"/><TextBlock x:Name="TitleText" VerticalAlignment="Center" FontWeight="SemiBold"/></StackPanel>
     <Button x:Name="XButton" Grid.Column="1" Content="×" Width="36" MinHeight="32" Padding="0" FontSize="20" Background="Transparent" BorderBrush="Transparent" VerticalAlignment="Center"/>
    </Grid>
   </Border>
   <Grid Grid.Row="1"><Grid.ColumnDefinitions><ColumnDefinition Width="210"/><ColumnDefinition Width="*"/></Grid.ColumnDefinitions>
    <Border Background="{DynamicResource Panel}" BorderBrush="{DynamicResource Line}" BorderThickness="0,0,1,0">
     <StackPanel Margin="23,30,23,26">
      <Image x:Name="Logo" Width="88" Height="88" HorizontalAlignment="Left" Margin="0,0,0,19" RenderOptions.BitmapScalingMode="HighQuality"/>
      <TextBlock Text="ReplyNoti" FontSize="25" FontWeight="Bold"/><TextBlock Text="v0.1.0-beta.11" FontSize="12" Foreground="{DynamicResource Muted}" Margin="0,7,0,0"/>
      <Border Height="1" Background="{DynamicResource Line}" Margin="0,24,0,22"/>
      <TextBlock x:Name="Unofficial" Foreground="{DynamicResource AccentText}" FontSize="13" FontWeight="SemiBold"/>
      <TextBlock x:Name="Privacy" Foreground="{DynamicResource Muted}" FontSize="12" Margin="0,10,0,0" LineHeight="19"/>
     </StackPanel>
    </Border>
    <StackPanel Grid.Column="1" Margin="30,26,30,22">
     <TextBlock x:Name="Step" FontSize="25" FontWeight="SemiBold"/>
     <TextBlock x:Name="Intro" Foreground="{DynamicResource Muted}" Margin="0,7,0,0" FontSize="13"/>
     <TextBlock x:Name="Options" FontWeight="SemiBold" Margin="0,25,0,10" FontSize="12" Foreground="{DynamicResource Muted}"/>
     <Border CornerRadius="10" Background="{DynamicResource Card}" BorderBrush="{DynamicResource Line}" BorderThickness="1">
      <StackPanel>
       <CheckBox x:Name="Startup" Margin="15,8"><StackPanel><TextBlock x:Name="StartupLabel" FontWeight="SemiBold"/><TextBlock x:Name="StartupDesc" Foreground="{DynamicResource Muted}" FontSize="12" Margin="0,4,0,0"/></StackPanel></CheckBox>
       <Border Height="1" Background="{DynamicResource Line}"/>
       <CheckBox x:Name="Preview" Margin="15,8"><StackPanel><TextBlock x:Name="PreviewLabel" FontWeight="SemiBold"/><TextBlock x:Name="PreviewDesc" Foreground="{DynamicResource Muted}" FontSize="12" Margin="0,4,0,0"/></StackPanel></CheckBox>
       <Border x:Name="PreviewDivider" Height="1" Background="{DynamicResource Line}"/>
       <CheckBox x:Name="Launch" Margin="15,8"><StackPanel><TextBlock x:Name="LaunchLabel" FontWeight="SemiBold"/><TextBlock x:Name="LaunchDesc" Foreground="{DynamicResource Muted}" FontSize="12" Margin="0,4,0,0"/></StackPanel></CheckBox>
      </StackPanel>
     </Border>
     <TextBlock x:Name="LocationLabel" FontSize="12" FontWeight="SemiBold" Foreground="{DynamicResource Muted}" Margin="0,19,0,7"/>
     <Border Background="{DynamicResource Input}" BorderBrush="{DynamicResource Line}" BorderThickness="1" CornerRadius="7" Padding="12,10"><TextBlock Text="%LOCALAPPDATA%\ReplyNoti" FontSize="13"/></Border>
     <TextBlock x:Name="LocationDesc" FontSize="12" Foreground="{DynamicResource Muted}" Margin="0,8,0,0"/>
     <TextBlock x:Name="Keep" FontSize="12" Foreground="{DynamicResource AccentText}" Margin="0,9,0,0"/>
     <StackPanel x:Name="StatusPanel" Visibility="Collapsed" Margin="0,15,0,0">
      <ProgressBar x:Name="Progress" Height="6" IsIndeterminate="True" Foreground="{DynamicResource Accent}" Background="{DynamicResource Input}"/>
      <TextBlock x:Name="Status" Margin="0,9,0,0" FontWeight="SemiBold"/>
      <TextBox x:Name="Details" Visibility="Collapsed" MaxHeight="90" Margin="0,8,0,0" Padding="7" Background="{DynamicResource Input}" Foreground="{DynamicResource Text}" BorderBrush="{DynamicResource Line}" TextWrapping="Wrap" VerticalScrollBarVisibility="Auto" IsReadOnly="True" FontSize="11"/>
     </StackPanel>
    </StackPanel>
   </Grid>
   <Border Grid.Row="2" Background="{DynamicResource Card}" BorderBrush="{DynamicResource Line}" BorderThickness="0,1,0,0" CornerRadius="0,0,11,11" Padding="22,13">
    <Grid><Grid.ColumnDefinitions><ColumnDefinition Width="*"/><ColumnDefinition Width="Auto"/></Grid.ColumnDefinitions>
     <TextBlock x:Name="Footer" Foreground="{DynamicResource Muted}" FontSize="11" VerticalAlignment="Center"/>
     <StackPanel Grid.Column="1" Orientation="Horizontal"><Button x:Name="Cancel" MinWidth="86" Margin="0,0,9,0"/><Button x:Name="Install" Style="{StaticResource Primary}" IsDefault="True"/></StackPanel>
    </Grid>
   </Border>
  </Grid>
 </Border>
</Window>
'@
$w=[Windows.Markup.XamlReader]::Parse($xaml)
if(!$dark){
 $palette=@{Bg='#FFFFFF';Panel='#F5F5F5';Card='#FFFFFF';Input='#F0F0F0';Text='#242424';Muted='#595959';Line='#DDDDDD';Accent='#2456D6';AccentText='#2456D6'}
 foreach($k in $palette.Keys){$w.Resources[$k]=[System.Windows.Media.SolidColorBrush]::new([System.Windows.Media.ColorConverter]::ConvertFromString($palette[$k]))}
}
$c=@{}
foreach($name in @('TitleBar','SmallIcon','Logo','TitleText','XButton','Step','Intro','Unofficial','Privacy','Options','Startup','StartupLabel','StartupDesc','Preview','PreviewLabel','PreviewDesc','PreviewDivider','Launch','LaunchLabel','LaunchDesc','LocationLabel','LocationDesc','Keep','StatusPanel','Progress','Status','Details','Footer','Cancel','Install')){$c[$name]=$w.FindName($name)}
$w.Title=$t.Title
$icon=[Windows.Media.Imaging.BitmapImage]::new()
$icon.BeginInit();$icon.CacheOption=[Windows.Media.Imaging.BitmapCacheOption]::OnLoad;$icon.UriSource=[Uri]::new((Join-Path $PSScriptRoot 'setup-icon.png'));$icon.EndInit();$icon.Freeze()
$w.Icon=$icon;$c.SmallIcon.Source=$icon;$c.Logo.Source=$icon
foreach($name in @('Step','Intro','Unofficial','Privacy','Options','StartupDesc','PreviewDesc','LaunchDesc','LocationDesc','Keep')){$c[$name].Text=$t[$name]}
$c.TitleText.Text=$t.Title
$c.StartupLabel.Text=$t.Startup;$c.PreviewLabel.Text=$t.Preview;$c.LaunchLabel.Text=$t.Launch;$c.LocationLabel.Text=$t.Location
foreach($n in @('Startup','Preview','Launch')){[Windows.Automation.AutomationProperties]::SetName($c[$n],$t[$n])}
[Windows.Automation.AutomationProperties]::SetName($c.XButton,$t.Close)
$c.Startup.IsChecked=$run -eq ('"'+(Join-Path $root 'replynoti.exe')+'" --run')
$c.Preview.IsChecked=$true
$c.Launch.IsChecked=!$upgrade
if($upgrade){$c.Preview.Visibility='Collapsed';$c.PreviewDivider.Visibility='Collapsed'}else{$c.Keep.Visibility='Collapsed'}
$c.Cancel.Content=$t.Cancel
$c.Install.Content=if($upgrade){$t.Update}else{$t.Install}
$c.Footer.Text='Windows 11 x64'
$script:phase='ready';$script:result=2;$script:child=$null
$stdout=Join-Path $PSScriptRoot 'setup-output.txt';$stderr=Join-Path $PSScriptRoot 'setup-error.txt'
$timer=[Windows.Threading.DispatcherTimer]::new()
$timer.Interval=[TimeSpan]::FromMilliseconds(200)
$timer.Add_Tick({
 if($null -eq $script:child -or !$script:child.HasExited){return}
 $timer.Stop();$script:child.WaitForExit();$code=$script:child.ExitCode;$script:child.Dispose();$script:child=$null
 $c.Progress.Visibility='Collapsed';$c.Cancel.IsEnabled=$true;$c.XButton.IsEnabled=$true;$c.Install.IsEnabled=$true
 if($code -eq 0){
  $script:phase='done';$script:result=0;$c.Status.Text=$t.Done;$c.Install.Content=$t.Close;$c.Cancel.Visibility='Collapsed';$c.Footer.Text='Windows 11 x64'
  if($c.Launch.IsChecked){try{Start-Process -FilePath (Join-Path $root 'replynoti.exe') -ArgumentList '--run' -WorkingDirectory $root}catch{$c.Details.Text=$_.Exception.Message;$c.Details.Visibility='Visible'}}
 }else{
  $script:phase='failed';$script:result=1;$c.Status.Text=$t.Failed;$c.Install.Content=$t.Retry;$c.Footer.Text=$t.ErrorHint
  $c.Details.Text=((Get-Content $stderr -Raw -ErrorAction SilentlyContinue)+[Environment]::NewLine+(Get-Content $stdout -Raw -ErrorAction SilentlyContinue)).Trim()
  $c.Details.Visibility='Visible';$c.Startup.IsEnabled=$true;$c.Preview.IsEnabled=$true;$c.Launch.IsEnabled=$true
 }
})
$c.Install.Add_Click({
 if($script:phase -eq 'done'){$w.Close();return}
 if($script:phase -eq 'working'){return}
 $script:phase='working';$c.StatusPanel.Visibility='Visible';$c.Progress.Visibility='Visible';$c.Details.Visibility='Collapsed';$c.Status.Text=$t.Working;$c.Footer.Text=$t.Hint
 foreach($n in @('Cancel','XButton','Install','Startup','Preview','Launch')){$c[$n].IsEnabled=$false}
 $arguments=@('-NoProfile','-NonInteractive','-ExecutionPolicy','RemoteSigned','-File',('"'+(Join-Path $PSScriptRoot 'Install.ps1')+'"'),'-Upgrade','-NoStart')
 if($c.Startup.IsChecked){$arguments+='-AutoStart'}else{$arguments+='-NoAutoStart'}
 if(!$upgrade -and !$c.Preview.IsChecked){$arguments+='-NoPreview'}
 try{
  $script:child=Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -ArgumentList $arguments -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
  # Retain the process handle: PowerShell can otherwise lose ExitCode after exit.
  $null=$script:child.Handle
  $timer.Start()
 }catch{
  $script:phase='failed';$script:result=1;$c.Progress.Visibility='Collapsed';$c.Status.Text=$t.Failed;$c.Details.Text=$_.Exception.Message;$c.Details.Visibility='Visible';$c.Install.Content=$t.Retry
  foreach($n in @('Cancel','XButton','Install','Startup','Preview','Launch')){$c[$n].IsEnabled=$true}
 }
})
$c.Cancel.Add_Click({$w.Close()})
$c.XButton.Add_Click({$w.Close()})
$c.TitleBar.Add_MouseLeftButtonDown({try{$w.DragMove()}catch{}})
$w.Add_PreviewKeyDown({if($_.Key -eq [Windows.Input.Key]::Escape -and $script:phase -ne 'working'){$w.Close();$_.Handled=$true}})
$w.Add_Closing({if($script:phase -eq 'working'){$_.Cancel=$true}})
$null=$w.ShowDialog()
exit $script:result
